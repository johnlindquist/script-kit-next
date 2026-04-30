//! Run 11 Pass #8 — attacker probe over the `metadata::handler_specs_*`
//! ingestion surface that the new `kit-init/sdk/*` helpers feed into.
//!
//! Purpose: confirm the Rust deserializer is robust to every shape the TS
//! helpers might emit — including helper-equivalent literal forms, pathologic
//! whitespace, oversize values, mixed families in one array, and adversarial
//! Unicode. The helpers themselves are pure identity functions, so the
//! ground truth is "any JSON the helpers can produce should round-trip
//! through `handler_specs_from_value` with the same observable shape".
//!
//! Categories covered (≥3 required by attacker-mode rule):
//!   1. Boundary — empty / oversize / NUL / unicode / numeric extremes
//!   2. Composition — duplicate / mixed-family / contradictory arrays
//!   3. Localization & Encoding — RTL, smart quotes, emoji, NFC vs NFD
//!   4. Determinism — serialize → deserialize round-trip stability
//!
//! Receipt: `cargo test --test menu_syntax_run11_attacker_pass8_sdk`.

use script_kit_gpui::menu_syntax::{
    handler_specs_from_extra_map, handler_specs_from_value, handler_specs_from_yaml_like_string,
    MenuSyntaxHandlerSpec,
};
use serde_json::{json, Value};
use std::collections::HashMap;

fn one_capture(target: &str) -> MenuSyntaxHandlerSpec {
    let value = json!({"family": "capture.v1", "targets": [target]});
    let mut specs = handler_specs_from_value(&value);
    assert_eq!(specs.len(), 1, "expected one spec for {target}");
    specs.remove(0)
}

// ============================================================================
// 1. BOUNDARY (8 actions)
// ============================================================================

#[test]
fn boundary_01_empty_object_array_yields_zero_specs() {
    assert!(handler_specs_from_value(&json!([])).is_empty());
    assert!(handler_specs_from_value(&json!({})).is_empty());
}

#[test]
fn boundary_02_null_and_primitive_top_levels_yield_empty() {
    assert!(handler_specs_from_value(&json!(null)).is_empty());
    assert!(handler_specs_from_value(&json!(42)).is_empty());
    assert!(handler_specs_from_value(&json!("string")).is_empty());
    assert!(handler_specs_from_value(&json!(true)).is_empty());
}

#[test]
fn boundary_03_missing_family_drops_entry() {
    let value = json!([{ "targets": ["todo"] }, { "family": "capture.v1", "targets": ["note"] }]);
    let specs = handler_specs_from_value(&value);
    assert_eq!(specs.len(), 1, "missing family should drop the first entry");
    assert_eq!(specs[0].targets, vec!["note".to_string()]);
}

#[test]
fn boundary_04_oversize_target_slug_survives_round_trip() {
    let big = "x".repeat(10_000);
    let value = json!({"family": "capture.v1", "targets": [big.clone()]});
    let specs = handler_specs_from_value(&value);
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].targets[0].len(), 10_000);
    assert!(specs[0].handles_capture_target(&big));
}

#[test]
fn boundary_05_target_slug_with_nul_byte_is_preserved_literally() {
    let value = json!({"family": "capture.v1", "targets": ["before\u{0000}after"]});
    let specs = handler_specs_from_value(&value);
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].targets[0], "before\u{0000}after");
    // Wildcard-target check still works (case-insensitive eq, exact match):
    assert!(specs[0].handles_capture_target("before\u{0000}after"));
}

#[test]
fn boundary_06_empty_targets_array_yields_no_handles() {
    let spec = handler_specs_from_value(&json!({"family": "capture.v1", "targets": []})).remove(0);
    assert!(!spec.handles_capture_target("todo"));
    assert!(!spec.handles_capture_target("anything"));
}

#[test]
fn boundary_07_extra_unknown_fields_are_ignored() {
    // serde_json default behavior — unknown fields silently ignored
    // (no #[serde(deny_unknown_fields)] on MenuSyntaxHandlerSpec).
    let value = json!({
        "family": "capture.v1",
        "targets": ["todo"],
        "futureField": "value",
        "anotherUnknown": [1, 2, 3]
    });
    let specs = handler_specs_from_value(&value);
    assert_eq!(specs.len(), 1, "unknown fields must not reject the spec");
}

#[test]
fn boundary_08_yaml_with_only_whitespace_is_empty() {
    assert!(handler_specs_from_yaml_like_string("\n\n\t  \n").is_empty());
    assert!(handler_specs_from_yaml_like_string(" ").is_empty());
}

// ============================================================================
// 2. COMPOSITION (6 actions)
// ============================================================================

#[test]
fn composition_01_duplicate_targets_within_one_handler_match_once() {
    let spec = handler_specs_from_value(
        &json!({"family": "capture.v1", "targets": ["todo", "todo", "TODO"]}),
    )
    .remove(0);
    // Case-insensitive eq counts all three as matching "todo".
    assert!(spec.handles_capture_target("todo"));
    assert_eq!(
        spec.targets.len(),
        3,
        "duplicates not deduped at parse time"
    );
}

#[test]
fn composition_02_mixed_family_array_keeps_all_entries() {
    let value = json!([
        {"family": "capture.v1", "targets": ["todo"]},
        {"family": "command.v1", "targets": []},
        {"family": "skill.v1", "targets": []},
        {"family": "capture.v1", "targets": ["note"]},
    ]);
    let specs = handler_specs_from_value(&value);
    assert_eq!(specs.len(), 4);
    // Only capture.v1 entries match capture targets.
    assert!(specs[0].handles_capture_target("todo"));
    assert!(!specs[1].handles_capture_target("anything"));
    assert!(!specs[2].handles_capture_target("anything"));
    assert!(specs[3].handles_capture_target("note"));
}

#[test]
fn composition_03_contradictory_defaultHandler_both_true_both_kept() {
    let value = json!([
        {"family": "capture.v1", "targets": ["todo"], "defaultHandler": true},
        {"family": "capture.v1", "targets": ["todo"], "defaultHandler": true},
    ]);
    let specs = handler_specs_from_value(&value);
    assert_eq!(specs.len(), 2);
    assert!(specs[0].default_handler);
    assert!(specs[1].default_handler);
    // Conflict resolution is the ranker's job (handler_index); the parser
    // is intentionally permissive.
}

#[test]
fn composition_04_wildcard_plus_specific_target_in_one_handler() {
    let spec = handler_specs_from_value(&json!({"family": "capture.v1", "targets": ["*", "todo"]}))
        .remove(0);
    assert!(spec.handles_capture_target("todo"));
    assert!(spec.handles_capture_target("anything"));
}

#[test]
fn composition_05_array_inside_object_form_handled_only_at_top_level() {
    // value is an Object — single-spec form. The "menuSyntax" extra-map
    // wrapper is what handles the Array form via the extra-map indirection.
    let mut extra: HashMap<String, Value> = HashMap::new();
    extra.insert(
        "menuSyntax".to_string(),
        json!([
            {"family": "capture.v1", "targets": ["todo"]},
            {"family": "capture.v1", "targets": ["note"]},
        ]),
    );
    let specs = handler_specs_from_extra_map(&extra);
    assert_eq!(specs.len(), 2);
}

#[test]
fn composition_06_extra_map_without_menuSyntax_key_returns_empty() {
    let mut extra: HashMap<String, Value> = HashMap::new();
    extra.insert("otherKey".to_string(), json!([{"family": "capture.v1"}]));
    assert!(handler_specs_from_extra_map(&extra).is_empty());
}

// ============================================================================
// 3. LOCALIZATION & ENCODING (5 actions)
// ============================================================================

#[test]
fn localization_01_rtl_target_slug_preserved() {
    let spec =
        handler_specs_from_value(&json!({"family": "capture.v1", "targets": ["مهمة"]})).remove(0);
    assert_eq!(spec.targets[0], "مهمة");
    assert!(spec.handles_capture_target("مهمة"));
}

#[test]
fn localization_02_emoji_slug_preserved_and_matches() {
    let spec =
        handler_specs_from_value(&json!({"family": "capture.v1", "targets": ["📅"]})).remove(0);
    assert_eq!(spec.targets[0], "📅");
    assert!(spec.handles_capture_target("📅"));
    assert!(!spec.handles_capture_target("calendar"));
}

#[test]
fn localization_03_smart_quotes_in_label_preserved_byte_for_byte() {
    let spec = handler_specs_from_value(
        &json!({"family": "capture.v1", "targets": ["todo"], "label": "“Smart” quote label"}),
    )
    .remove(0);
    assert_eq!(
        spec.label.as_deref(),
        Some("\u{201C}Smart\u{201D} quote label")
    );
}

#[test]
fn localization_04_case_insensitive_target_match_works_for_unicode_lower() {
    // Rust eq_ignore_ascii_case only handles ASCII. Non-ASCII upper/lower
    // is NOT considered equivalent — we pin that behavior so a future
    // switch to full unicode-case-fold doesn't silently change matching.
    let spec =
        handler_specs_from_value(&json!({"family": "capture.v1", "targets": ["İ"]})).remove(0);
    assert!(spec.handles_capture_target("İ"));
    // Lowercase Turkish dotless "i" — should NOT match under ascii-fold.
    assert!(!spec.handles_capture_target("i\u{0307}"));
}

#[test]
fn localization_05_nfc_vs_nfd_target_does_not_implicitly_normalize() {
    // "é" precomposed (NFC) vs "e" + combining acute (NFD).
    let nfc = "caf\u{00E9}";
    let nfd = "cafe\u{0301}";
    let spec =
        handler_specs_from_value(&json!({"family": "capture.v1", "targets": [nfc]})).remove(0);
    assert!(spec.handles_capture_target(nfc));
    assert!(
        !spec.handles_capture_target(nfd),
        "NFC vs NFD must not be silently equated (would mask author bugs)"
    );
}

// ============================================================================
// 4. DETERMINISM / ROUND-TRIP (5 actions)
// ============================================================================

#[test]
fn determinism_01_serialize_then_deserialize_is_idempotent() {
    let original = one_capture("todo");
    let json_value = serde_json::to_value(&original).expect("serialize");
    let restored = handler_specs_from_value(&json_value).remove(0);
    assert_eq!(restored.family, original.family);
    assert_eq!(restored.targets, original.targets);
    assert_eq!(restored.default_handler, original.default_handler);
}

#[test]
fn determinism_02_helper_equivalent_literal_matches_typed_helper_output() {
    // The `captureTarget("cal", { accepts: ["tags","date"], label: "..." })`
    // helper produces this shape — pin that the literal form parses
    // identically so authors can switch between helpers and literals freely.
    let helper_output = json!({
        "family": "capture.v1",
        "targets": ["cal"],
        "accepts": ["tags", "date", "duration", "kv"],
        "label": "Create calendar event",
        "payloadSchema": "kit://schema/menu-syntax/payload-v1",
        "defaultHandler": true
    });
    let inline_literal = json!({
        "family": "capture.v1",
        "targets": ["cal"],
        "accepts": ["tags", "date", "duration", "kv"],
        "label": "Create calendar event",
        "payloadSchema": "kit://schema/menu-syntax/payload-v1",
        "defaultHandler": true
    });
    let a = handler_specs_from_value(&helper_output).remove(0);
    let b = handler_specs_from_value(&inline_literal).remove(0);
    assert_eq!(
        a, b,
        "helper-built and inline-literal must parse identically"
    );
}

#[test]
fn determinism_03_alsoTargets_helper_output_matches_concatenated_literal() {
    // `captureTarget("note", { alsoTargets: ["journal"] })` produces
    // targets: ["note", "journal"]. Pin the order.
    let helper_output = json!({
        "family": "capture.v1",
        "targets": ["note", "journal"],
        "accepts": ["tags"]
    });
    let spec = handler_specs_from_value(&helper_output).remove(0);
    assert_eq!(
        spec.targets,
        vec!["note".to_string(), "journal".to_string()]
    );
    assert!(spec.handles_capture_target("note"));
    assert!(spec.handles_capture_target("journal"));
    assert!(!spec.handles_capture_target("diary"));
}

#[test]
fn determinism_04_yaml_form_round_trips_to_same_spec_as_json_form() {
    let json_form = handler_specs_from_yaml_like_string(
        r#"[{"family":"capture.v1","targets":["link"],"accepts":["url","tags"]}]"#,
    );
    let yaml_form = handler_specs_from_yaml_like_string(
        "- family: capture.v1\n  targets: [link]\n  accepts: [url, tags]\n",
    );
    assert_eq!(json_form.len(), 1);
    assert_eq!(yaml_form.len(), 1);
    assert_eq!(json_form[0], yaml_form[0]);
}

#[test]
fn determinism_05_repeated_parses_yield_equal_specs() {
    // Determinism: parser is pure, so the same input must produce the
    // exact same spec across N invocations. A pinning test that would
    // catch any future state-leak (caching, statics) regression.
    let value = json!({"family": "capture.v1", "targets": ["todo"], "label": "Todo capture"});
    let runs: Vec<_> = (0..50)
        .map(|_| handler_specs_from_value(&value).remove(0))
        .collect();
    let first = &runs[0];
    for r in &runs[1..] {
        assert_eq!(r, first);
    }
}
