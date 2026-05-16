// doc-anchor-removed: [[removed-docs Syntax#Capture Validation Gate]]
//
// Pure decision module that branches Enter on a `+target` invocation into
// Allow / Block based on the resolved [[src/menu_syntax/capture_schema.rs#CaptureFieldSchema]].
// Schema lookup goes builtin-first, then dynamic via the script's matching
// `capture.v1` spec. When no schema is known the gate is permissive (Allow):
// silence is preserved for handlers that opt out of declared shape.

use std::sync::Arc;

use crate::menu_syntax::capture_schema::{
    builtin_schema, validate, CaptureFieldSchema, FieldRequirement, ValidationResult,
};
use crate::menu_syntax::filter::script_menu_syntax_specs;
use crate::menu_syntax::metadata::dynamic_capture_schema_from_spec;
use crate::menu_syntax::payload::{CaptureInvocation, DatePhrase};
use crate::scripts::Script;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureGateDecision {
    Allow,
    BlockMissing {
        hud_message: String,
        missing: Vec<FieldRequirement>,
    },
    BlockMalformed {
        hud_message: String,
        field: FieldRequirement,
        reason: String,
    },
}

impl CaptureGateDecision {
    pub fn is_allow(&self) -> bool {
        matches!(self, CaptureGateDecision::Allow)
    }

    /// Run 12 Pass 14 — accessor for the HUD nudge so the gate decision
    /// is observable through the snapshot (`captureValidation.hudMessage`)
    /// rather than only via the side-effect `show_hud` call. Returns
    /// `None` for `Allow`; the live HUD copy for the two block variants.
    pub fn hud_message(&self) -> Option<&str> {
        match self {
            CaptureGateDecision::Allow => None,
            CaptureGateDecision::BlockMissing { hud_message, .. }
            | CaptureGateDecision::BlockMalformed { hud_message, .. } => Some(hud_message.as_str()),
        }
    }
}

/// Run 12 Pass 15 — resolve a capture schema for a `+target` ahead of any
/// script-handler match. Used by the snapshot layer
/// (`capture_validation_chips_and_snapshot`) so the live UI sees
/// dynamic schemas declared by ANY loaded script that registers a
/// `capture.v1` spec for this target. Builtin schema wins when one
/// exists; otherwise the first matching dynamic schema is returned.
/// Returns `None` for unknown custom targets with no script handler.
pub fn resolve_capture_schema_for_target(
    target: &str,
    scripts: &[std::sync::Arc<Script>],
) -> Option<CaptureFieldSchema> {
    if let Some(schema) = builtin_schema(target) {
        return Some(schema);
    }
    for script in scripts {
        for spec in script_menu_syntax_specs(script) {
            if spec.family != "capture.v1" {
                continue;
            }
            let matches_target = spec
                .targets
                .iter()
                .any(|t| t == "*" || t.eq_ignore_ascii_case(target));
            if !matches_target {
                continue;
            }
            if let Some(schema) = dynamic_capture_schema_from_spec(&spec) {
                return Some(schema);
            }
        }
    }
    None
}

pub fn resolve_capture_schema_for_script(
    invocation: &CaptureInvocation,
    script: &Script,
) -> Option<CaptureFieldSchema> {
    if let Some(schema) = builtin_schema(&invocation.target) {
        return Some(schema);
    }
    for spec in script_menu_syntax_specs(script) {
        if spec.family != "capture.v1" {
            continue;
        }
        let matches_target = spec
            .targets
            .iter()
            .any(|t| t == "*" || t.eq_ignore_ascii_case(&invocation.target));
        if !matches_target {
            continue;
        }
        if let Some(schema) = dynamic_capture_schema_from_spec(&spec) {
            return Some(schema);
        }
    }
    None
}

pub fn decide_capture_gate(
    invocation: &CaptureInvocation,
    schema: Option<&CaptureFieldSchema>,
) -> CaptureGateDecision {
    decide_capture_gate_with_accepts(invocation, schema, &[])
}

pub fn decide_capture_gate_with_accepts(
    invocation: &CaptureInvocation,
    schema: Option<&CaptureFieldSchema>,
    accepts: &[String],
) -> CaptureGateDecision {
    let Some(schema) = schema else {
        return CaptureGateDecision::Allow;
    };
    let probed;
    let validation_invocation = if should_probe_nl_any_date(invocation, schema, accepts) {
        probed = invocation_with_resolved_dates(invocation, accepts);
        probed.as_ref().unwrap_or(invocation)
    } else {
        invocation
    };
    match validate(validation_invocation, schema) {
        ValidationResult::Ready => CaptureGateDecision::Allow,
        ValidationResult::Incomplete { missing } => {
            let labels: Vec<String> = missing.iter().map(FieldRequirement::label).collect();
            // Target-aware HUD nudge with a fix-it example pulled from the
            // shared `target_examples` source so the suggestion matches the
            // hint card's example list. Run 12 Pass 3 user priority #2.
            let example = crate::menu_syntax::main_hint::target_examples(&invocation.target)
                .into_iter()
                .next()
                .unwrap_or_else(|| format!(";{} body text", invocation.target));
            let hud = format!(
                ";{} needs {} — try `{}`",
                invocation.target,
                join_oxford(&labels),
                example
            );
            CaptureGateDecision::BlockMissing {
                hud_message: hud,
                missing,
            }
        }
        ValidationResult::Malformed { field, reason } => {
            let hud = format!(";{}: {}", invocation.target, reason);
            CaptureGateDecision::BlockMalformed {
                hud_message: hud,
                field,
                reason,
            }
        }
    }
}

pub fn decide_capture_gate_for_script(
    invocation: &CaptureInvocation,
    script: &Arc<Script>,
) -> CaptureGateDecision {
    let schema = resolve_capture_schema_for_script(invocation, script);
    let Some(schema) = schema else {
        return CaptureGateDecision::Allow;
    };
    let accepts = script_menu_syntax_specs(script)
        .into_iter()
        .find(|spec| spec.handles_capture_target(&invocation.target))
        .map(|spec| spec.accepts)
        .unwrap_or_default();
    decide_capture_gate_with_accepts(invocation, Some(&schema), &accepts)
}

fn should_probe_nl_any_date(
    invocation: &CaptureInvocation,
    schema: &CaptureFieldSchema,
    accepts: &[String],
) -> bool {
    invocation.date_phrases.is_empty()
        && schema
            .required
            .iter()
            .any(|field| matches!(field, FieldRequirement::AnyDate))
        && (invocation.target.eq_ignore_ascii_case("cal")
            || invocation.target.eq_ignore_ascii_case("mcal")
            || accepts.iter().any(|accept| {
                matches!(
                    accept.as_str(),
                    "date"
                        | "dateRange"
                        | "duration"
                        | "recurrence"
                        | "relativeDate"
                        | "daily"
                        | "multiWeekday"
                        | "monthly"
                        | "yearly"
                )
            }))
}

fn invocation_with_resolved_dates(
    invocation: &CaptureInvocation,
    accepts: &[String],
) -> Option<CaptureInvocation> {
    let clock = crate::menu_syntax::date::MenuSyntaxClock::local_now();
    let resolved =
        crate::menu_syntax::date::resolve_capture_dates_with_accepts(invocation, &clock, accepts);
    let mut probed = invocation.clone();
    probed.body = resolved.body;
    probed.duration = resolved.duration;
    probed.date_phrases = resolved
        .dates
        .into_iter()
        .map(|date| DatePhrase {
            role: date.role,
            source: date.source,
            source_span: date.source_span,
        })
        .collect();
    if probed.body != invocation.body
        || probed.duration != invocation.duration
        || probed.date_phrases != invocation.date_phrases
    {
        Some(probed)
    } else {
        None
    }
}

fn join_oxford(labels: &[String]) -> String {
    match labels.len() {
        0 => String::new(),
        1 => labels[0].clone(),
        2 => format!("{} and {}", labels[0], labels[1]),
        _ => {
            let head = labels[..labels.len() - 1].join(", ");
            format!("{}, and {}", head, labels[labels.len() - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::payload::{CaptureAlias, DatePhrase, DateRole};

    fn empty_inv(target: &str) -> CaptureInvocation {
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
            raw: format!(";{}", target),
        }
    }

    fn cal_complete() -> CaptureInvocation {
        let mut inv = empty_inv("cal");
        inv.body = "Design review".to_string();
        inv.date_phrases.push(DatePhrase {
            role: DateRole::Inferred,
            source: "friday 2pm".to_string(),
            source_span: (0, 10),
        });
        inv
    }

    #[test]
    fn no_schema_allows() {
        let inv = empty_inv("custom-handler-target");
        let decision = decide_capture_gate(&inv, None);
        assert_eq!(decision, CaptureGateDecision::Allow);
    }

    #[test]
    fn cal_with_no_body_or_date_blocks_with_both_labels() {
        let inv = empty_inv("cal");
        let schema = builtin_schema("cal").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        match decision {
            CaptureGateDecision::BlockMissing {
                hud_message,
                missing,
            } => {
                assert_eq!(missing.len(), 2);
                assert!(hud_message.contains("body"), "{hud_message}");
                assert!(hud_message.contains("date"), "{hud_message}");
                assert!(hud_message.starts_with(";cal needs "));
            }
            other => panic!("expected BlockMissing, got {other:?}"),
        }
    }

    #[test]
    fn cal_complete_allows() {
        let inv = cal_complete();
        let schema = builtin_schema("cal").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        assert_eq!(decision, CaptureGateDecision::Allow);
        assert!(decision.is_allow());
    }

    #[test]
    fn link_with_malformed_url_blocks_malformed() {
        let mut inv = empty_inv("link");
        inv.url = Some("ftp://nope".to_string());
        let schema = builtin_schema("link").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        match decision {
            CaptureGateDecision::BlockMalformed {
                hud_message, field, ..
            } => {
                assert_eq!(field, FieldRequirement::Url);
                assert!(hud_message.starts_with(";link:"));
                assert!(hud_message.contains("URL"));
            }
            other => panic!("expected BlockMalformed, got {other:?}"),
        }
    }

    #[test]
    fn todo_with_body_allows() {
        let mut inv = empty_inv("todo");
        inv.body = "buy milk".to_string();
        let schema = builtin_schema("todo").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        assert_eq!(decision, CaptureGateDecision::Allow);
    }

    #[test]
    fn join_oxford_handles_one_two_three() {
        assert_eq!(join_oxford(&["body".into()]), "body");
        assert_eq!(
            join_oxford(&["body".into(), "date".into()]),
            "body and date"
        );
        assert_eq!(
            join_oxford(&["body".into(), "date".into(), "url".into()]),
            "body, date, and url"
        );
    }

    #[test]
    fn missing_single_field_no_oxford_comma() {
        let inv = empty_inv("note");
        let schema = builtin_schema("note").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        match decision {
            CaptureGateDecision::BlockMissing { hud_message, .. } => {
                // Run 12 Pass 3: the HUD now includes a fix-it example after
                // the "needs <fields>" prefix; assertions check the prefix +
                // example shape rather than equality with the bare prefix.
                assert!(
                    hud_message.starts_with(";note needs body"),
                    "expected ;note needs body prefix, got {hud_message}"
                );
                assert!(
                    hud_message.contains("— try `;note "),
                    "expected fix-it example, got {hud_message}"
                );
            }
            other => panic!("expected BlockMissing, got {other:?}"),
        }
    }

    #[test]
    fn mcal_with_nl_range_satisfies_any_date() {
        let mut inv = empty_inv("mcal");
        inv.body = "Lunch tomorrow 12pm til 1pm".to_string();
        let schema = builtin_schema("mcal").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        assert_eq!(decision, CaptureGateDecision::Allow);
    }

    #[test]
    fn dynamic_date_range_accepts_nl_date_satisfies_any_date() {
        let mut inv = empty_inv("appt");
        inv.body = "Lunch tomorrow at 12pm til 1pm".to_string();
        let schema = CaptureFieldSchema {
            target: "appt".to_string(),
            required: vec![FieldRequirement::Body, FieldRequirement::AnyDate],
            optional: vec![],
            forbidden: vec![],
        };
        let accepts = vec!["dateRange".to_string()];
        let decision = decide_capture_gate_with_accepts(&inv, Some(&schema), &accepts);
        assert_eq!(decision, CaptureGateDecision::Allow);
    }

    #[test]
    fn snooze_with_relative_date_satisfies_any_date() {
        let mut inv = empty_inv("snooze");
        inv.body = "in 30 minutes".to_string();
        let schema = CaptureFieldSchema {
            target: "snooze".to_string(),
            required: vec![FieldRequirement::AnyDate],
            optional: vec![],
            forbidden: vec![],
        };
        let accepts = vec!["relativeDate".to_string()];
        let decision = decide_capture_gate_with_accepts(&inv, Some(&schema), &accepts);
        assert_eq!(decision, CaptureGateDecision::Allow);
    }

    #[test]
    fn mcal_date_only_blocks_body_after_nl_date_probe() {
        let mut inv = empty_inv("mcal");
        inv.body = "tomorrow at 12pm til 1pm".to_string();
        let schema = builtin_schema("mcal").unwrap();
        let accepts = vec!["dateRange".to_string()];
        let decision = decide_capture_gate_with_accepts(&inv, Some(&schema), &accepts);
        match decision {
            CaptureGateDecision::BlockMissing { missing, .. } => {
                assert_eq!(missing, vec![FieldRequirement::Body]);
            }
            other => panic!("expected BlockMissing body only, got {other:?}"),
        }
    }

    #[test]
    fn mcal_without_nl_date_still_blocks_missing_date() {
        let mut inv = empty_inv("mcal");
        inv.body = "Lunch with Ryan".to_string();
        let schema = builtin_schema("mcal").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        match decision {
            CaptureGateDecision::BlockMissing { missing, .. } => {
                assert!(missing.contains(&FieldRequirement::AnyDate));
            }
            other => panic!("expected BlockMissing, got {other:?}"),
        }
    }

    // -------- Run 12 Pass 3 — capture-required-fields-block-enter-with-nudge --------

    #[test]
    fn cal_block_hud_includes_fix_it_example_with_date_slot() {
        // Falsifier: HUD must include a fix-it example pulled from the
        // shared `target_examples` source. The +cal example MUST contain a
        // date slot so the user can paste-and-edit it to fix the gate.
        let inv = empty_inv("cal");
        let schema = builtin_schema("cal").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        match decision {
            CaptureGateDecision::BlockMissing { hud_message, .. } => {
                assert!(
                    hud_message.contains("— try `;cal "),
                    ";cal HUD must include a ;cal fix-it example, got {hud_message}"
                );
                assert!(
                    hud_message.contains("start:")
                        || hud_message.contains("at:")
                        || hud_message.contains("due:")
                        || hud_message.contains("end:"),
                    ";cal fix-it example must include a date slot, got {hud_message}"
                );
            }
            other => panic!("expected BlockMissing, got {other:?}"),
        }
    }

    #[test]
    fn fix_it_example_uses_target_verb_no_cross_target_leakage() {
        // Falsifier: the fix-it example for `+todo` must use `+todo`, not
        // `+cal` or any other verb. Same invariant as the hint card's
        // `target_examples_for_*` tests but at the gate boundary.
        let inv = empty_inv("todo");
        let schema = builtin_schema("todo").unwrap();
        let decision = decide_capture_gate(&inv, Some(&schema));
        match decision {
            CaptureGateDecision::BlockMissing { hud_message, .. } => {
                assert!(hud_message.contains("— try `;todo "), "got {hud_message}");
                assert!(
                    !hud_message.contains("`+cal "),
                    ";todo HUD must not leak ;cal example, got {hud_message}"
                );
            }
            other => panic!("expected BlockMissing, got {other:?}"),
        }
    }

    #[test]
    fn fix_it_example_for_unknown_target_falls_back_to_target_verb() {
        // Custom user-defined targets get the generic example list per
        // `target_examples`'s fallback arm. The fix-it must still use the
        // user's verb — no `+todo` leakage on the gate path either.
        let mut schema = builtin_schema("todo").unwrap();
        schema.target = "expense".to_string();
        let mut inv = empty_inv("expense");
        inv.body = String::new();
        let decision = decide_capture_gate(&inv, Some(&schema));
        match decision {
            CaptureGateDecision::BlockMissing { hud_message, .. } => {
                assert!(
                    hud_message.contains("— try `;expense "),
                    ";expense HUD must use ;expense fix-it, got {hud_message}"
                );
            }
            other => panic!("expected BlockMissing, got {other:?}"),
        }
    }
}
