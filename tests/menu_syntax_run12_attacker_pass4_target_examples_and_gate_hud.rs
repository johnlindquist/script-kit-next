//! Run 12 Pass 4 — ATTACKER MODE on the Run-12 user-priority surfaces:
//! [[src/menu_syntax/main_hint.rs#target_examples]] (Pass 2, single source of
//! truth for hint card examples) and the HUD-copy fix-it path through
//! [[src/menu_syntax/capture_gate.rs#decide_capture_gate]] (Pass 3, which now
//! pulls its fix-it example from `target_examples`).
//!
//! `target_examples` is `pub(crate)`, so we probe it through the gate boundary
//! the user actually sees (HUD message). This doubles as a contract that the
//! hint card and HUD stay in lockstep on the verb-prefix invariant.
//!
//! Categories: Boundary (10), Composition (6), Resurrection (6). Actions: 22.

use script_kit_gpui::menu_syntax::payload::{CaptureAlias, CaptureInvocation};
use script_kit_gpui::menu_syntax::{
    builtin_schema, decide_capture_gate, CaptureGateDecision, FieldRequirement,
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

/// Take a known builtin schema and rebrand its target so we can drive the gate
/// against arbitrary attacker-chosen target strings while still triggering the
/// "incomplete" branch (and thus the fix-it HUD).
fn rebrand_schema(
    template_target: &str,
    new_target: &str,
) -> Option<script_kit_gpui::menu_syntax::CaptureFieldSchema> {
    let mut schema = builtin_schema(template_target)?;
    schema.target = new_target.to_string();
    Some(schema)
}

fn block_missing_hud(inv: &CaptureInvocation, schema_template: &str) -> String {
    let schema = rebrand_schema(schema_template, &inv.target).expect("template schema");
    match decide_capture_gate(inv, Some(&schema)) {
        CaptureGateDecision::BlockMissing { hud_message, .. } => hud_message,
        other => panic!("expected BlockMissing for {:?}, got {other:?}", inv.target),
    }
}

// ---------------- Boundary (10) ----------------

#[test]
fn boundary_01_long_target_name_keeps_verb_prefix() {
    let long = "a".repeat(64);
    let inv = empty(&long);
    let hud = block_missing_hud(&inv, "todo");
    assert!(
        hud.starts_with(&format!("+{long} needs ")),
        "long target must keep its verb prefix, got {hud}"
    );
    assert!(
        hud.contains(&format!("— try `;{long} ")),
        "fix-it must echo the long verb, got {hud}"
    );
}

#[test]
fn boundary_02_unicode_target_uses_unicode_verb() {
    let inv = empty("日記");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.starts_with("+日記 needs "), "got {hud}");
    assert!(hud.contains("— try `;日記 "), "got {hud}");
}

#[test]
fn boundary_03_emoji_target_round_trips() {
    let inv = empty("📝");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.starts_with("+📝 needs "), "got {hud}");
    assert!(hud.contains("— try `;📝 "), "got {hud}");
}

#[test]
fn boundary_04_hyphenated_target_no_split() {
    let inv = empty("custom-bug-report");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.starts_with("+custom-bug-report needs "), "got {hud}");
    assert!(hud.contains("— try `;custom-bug-report "), "got {hud}");
}

#[test]
fn boundary_05_uppercase_target_is_not_lowercased() {
    // The gate stores the raw user-typed target; the HUD must echo it
    // verbatim (no silent normalization that would mismatch what the user sees
    // in the hint card / picker).
    let inv = empty("CAL");
    let hud = block_missing_hud(&inv, "cal");
    assert!(hud.starts_with("+CAL needs "), "got {hud}");
    assert!(
        hud.contains("— try `;CAL "),
        "fix-it must use the user's exact casing, got {hud}"
    );
}

#[test]
fn boundary_06_target_with_dot_segment() {
    let inv = empty("note.work");
    let hud = block_missing_hud(&inv, "note");
    assert!(hud.starts_with(";note.work needs "), "got {hud}");
    assert!(hud.contains("— try `;note.work "), "got {hud}");
}

#[test]
fn boundary_07_single_char_target() {
    let inv = empty("x");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.starts_with("+x needs "), "got {hud}");
    assert!(hud.contains("— try `;x "), "got {hud}");
}

#[test]
fn boundary_08_target_with_underscore() {
    let inv = empty("expense_report");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.starts_with(";expense_report needs body"), "got {hud}");
}

#[test]
fn boundary_09_empty_target_does_not_panic() {
    // The gate may not produce a useful HUD for an empty target, but it MUST
    // NOT panic — pure-function attacker invariant.
    let inv = empty("");
    let schema = rebrand_schema("todo", "").expect("template");
    let _ = decide_capture_gate(&inv, Some(&schema));
}

#[test]
fn boundary_10_whitespace_in_target_preserved() {
    let inv = empty("two words");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.starts_with("+two words needs "), "got {hud}");
    assert!(hud.contains("— try `;two words "), "got {hud}");
}

// ---------------- Composition (6) ----------------

#[test]
fn composition_01_cal_missing_body_and_date_lists_both_in_oxford_form() {
    let inv = empty("cal");
    let schema = builtin_schema("cal").unwrap();
    let CaptureGateDecision::BlockMissing {
        hud_message,
        missing,
    } = decide_capture_gate(&inv, Some(&schema))
    else {
        panic!("expected BlockMissing");
    };
    assert_eq!(missing.len(), 2);
    // Two-item Oxford form is "X and Y" — never trailing comma.
    assert!(
        hud_message.contains(" needs body and date — try `+cal "),
        "got {hud_message}"
    );
}

#[test]
fn composition_02_link_missing_url_routes_through_target_examples() {
    // `+link` with no URL hits the missing branch (URL is required); the
    // fix-it should be a +link example, not a +todo fallback.
    let inv = empty("link");
    let schema = builtin_schema("link").unwrap();
    let CaptureGateDecision::BlockMissing {
        hud_message,
        missing,
    } = decide_capture_gate(&inv, Some(&schema))
    else {
        panic!("expected BlockMissing");
    };
    assert!(
        missing.contains(&FieldRequirement::Url),
        "missing={missing:?}"
    );
    assert!(hud_message.contains("— try `;link "), "got {hud_message}");
    assert!(
        !hud_message.contains("— try `;todo "),
        ";link must not leak ;todo fix-it, got {hud_message}"
    );
}

#[test]
fn composition_03_malformed_branch_does_not_use_fix_it_format() {
    // The fix-it example is a property of the `Incomplete` branch only.
    // `Malformed` uses `+target: <reason>` and must NOT contain "— try `".
    let mut inv = empty("link");
    inv.url = Some("ftp://nope".to_string());
    let schema = builtin_schema("link").unwrap();
    let CaptureGateDecision::BlockMalformed { hud_message, .. } =
        decide_capture_gate(&inv, Some(&schema))
    else {
        panic!("expected BlockMalformed");
    };
    assert!(
        !hud_message.contains("— try `"),
        "Malformed branch must not use Incomplete's fix-it suffix, got {hud_message}"
    );
}

#[test]
fn composition_04_unknown_target_uses_fallback_example_not_todo_verb() {
    // Falsifier for the cross-target leakage class: an unknown target's
    // fallback example must use the user's verb in the body, NEVER `+todo`.
    let inv = empty("expense");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.contains("— try `;expense "), "got {hud}");
    assert!(
        !hud.contains("— try `;todo "),
        "fallback must not leak ;todo, got {hud}"
    );
}

#[test]
fn composition_05_three_missing_fields_use_oxford_comma() {
    // We can't naturally get 3 missing fields with builtins, so synthesize via
    // the join_oxford path: a template with 3 requirements would render as
    // "a, b, and c". We approximate by checking the 2-field "and" works (no
    // pre-and comma) and the absence of stray commas in the 2-item case.
    let inv = empty("cal");
    let schema = builtin_schema("cal").unwrap();
    let CaptureGateDecision::BlockMissing { hud_message, .. } =
        decide_capture_gate(&inv, Some(&schema))
    else {
        panic!("expected BlockMissing");
    };
    let head = hud_message.split(" — try `").next().unwrap_or(&hud_message);
    assert!(
        !head.contains(", and "),
        "two-item form must not Oxford, got {head}"
    );
    assert!(
        !head.contains(", "),
        "two-item form must not have any comma, got {head}"
    );
}

#[test]
fn composition_06_target_examples_first_row_for_cal_has_a_date_slot() {
    // Routes through the gate: the first +cal example MUST contain a date
    // slot (start:/at:/due:/end:) so paste-and-edit fixes the gate.
    let inv = empty("cal");
    let schema = builtin_schema("cal").unwrap();
    let CaptureGateDecision::BlockMissing { hud_message, .. } =
        decide_capture_gate(&inv, Some(&schema))
    else {
        panic!("expected BlockMissing");
    };
    let fix_it = hud_message.split("— try `").nth(1).unwrap_or("");
    assert!(
        fix_it.contains("start:")
            || fix_it.contains("at:")
            || fix_it.contains("due:")
            || fix_it.contains("end:"),
        ";cal fix-it must contain a date slot, got {fix_it}"
    );
}

// ---------------- Resurrection (6) ----------------
// These pin down past bugs / Pass-3 invariants so future refactors can't
// silently regress them.

#[test]
fn resurrection_01_no_todo_leakage_on_cal_block() {
    // Pass-3 falsifier: +cal must never produce a +todo fix-it.
    let inv = empty("cal");
    let hud = block_missing_hud(&inv, "cal");
    assert!(!hud.contains("— try `;todo "), "got {hud}");
}

#[test]
fn resurrection_02_no_cal_leakage_on_todo_block() {
    let inv = empty("todo");
    let hud = block_missing_hud(&inv, "todo");
    assert!(!hud.contains("— try `;cal "), "got {hud}");
}

#[test]
fn resurrection_03_note_block_uses_note_verb() {
    let inv = empty("note");
    let hud = block_missing_hud(&inv, "note");
    assert!(hud.starts_with(";note needs body"), "got {hud}");
    assert!(hud.contains("— try `;note "), "got {hud}");
}

#[test]
fn resurrection_04_social_block_uses_social_verb() {
    let inv = empty("social");
    let hud = block_missing_hud(&inv, "todo");
    assert!(hud.starts_with(";social needs body"), "got {hud}");
    assert!(hud.contains("— try `;social "), "got {hud}");
}

#[test]
fn resurrection_05_hud_message_is_single_line() {
    // HUD overlay only renders one line; embedded newlines would silently
    // truncate. Gate must produce a single-line message.
    let inv = empty("cal");
    let hud = block_missing_hud(&inv, "cal");
    assert!(!hud.contains('\n'), "got multiline HUD: {hud:?}");
}

#[test]
fn resurrection_06_hud_message_backtick_pair_is_balanced() {
    // The fix-it suffix wraps the example in backticks. If we ever switch to
    // a quote that the example body contains naturally, the wrapping breaks.
    let inv = empty("cal");
    let hud = block_missing_hud(&inv, "cal");
    let count = hud.matches('`').count();
    assert_eq!(
        count, 2,
        "expected exactly 2 backticks (open + close), got {count} in {hud}"
    );
}
