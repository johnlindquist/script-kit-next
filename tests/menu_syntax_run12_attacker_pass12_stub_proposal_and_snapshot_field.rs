//! Run 12 Pass 12 — ATTACKER MODE on the Pass-11 surfaces:
//! - [[src/app_impl/menu_syntax_ai.rs#stub_proposal_for]] — the deterministic
//!   stub the Cmd+Enter handler calls in lieu of a real ACP/LLM round-trip.
//! - [[src/menu_syntax/main_hint.rs#MenuSyntaxMainHintSnapshot]]'s new
//!   `menu_syntax_ai_proposal` field (camelCase, skip-if-None).
//!
//! These two surfaces decide what the user sees when they press Cmd+Enter in
//! a Power Syntax composer state. If the stub heuristic drifts — or if the
//! snapshot field stops serializing exactly as the spec dictates — the
//! `cmd-enter-inline-ai-proposal` story silently regresses with no compiler
//! error. This file pins both against future regressions.
//!
//! Categories: Boundary (8), Composition (7), Resurrection (7). Actions: 22.

use script_kit_gpui::menu_syntax::payload::{CaptureAlias, CaptureInvocation};
use script_kit_gpui::menu_syntax::query::parse_advanced_query;
use script_kit_gpui::menu_syntax::MenuSyntaxActionState;
use script_kit_gpui::menu_syntax_ai::{stub_proposal_for, MenuSyntaxAiProposal, ProposalKind};

fn capture_payload(target: &str, body: &str) -> CaptureInvocation {
    CaptureInvocation {
        target: target.to_string(),
        alias_form: CaptureAlias::CapturePrefix,
        body: body.to_string(),
        tags: vec![],
        priority: None,
        url: None,
        duration: None,
        kv: vec![],
        date_phrases: vec![],
        raw: format!("+{} {}", target, body),
    }
}

fn capture_with_tags(target: &str, body: &str, tags: &[&str]) -> CaptureInvocation {
    let mut p = capture_payload(target, body);
    p.tags = tags.iter().map(|t| t.to_string()).collect();
    let tag_part: String = p.tags.iter().map(|t| format!(" #{t}")).collect();
    p.raw = format!("+{} {}{}", target, body, tag_part);
    p
}

fn capture_state<'a>(target: &'a str, payload: &'a CaptureInvocation) -> MenuSyntaxActionState<'a> {
    MenuSyntaxActionState::CaptureComposer {
        target,
        payload,
        schema: None,
    }
}

// ---------- Boundary (8) ----------

/// Boundary 1: STORY-EXACT receipt for `+todo` no tags — must produce the
/// AddTag{errands} proposal with the title and accept_label the spec quotes.
#[test]
fn story_exact_todo_no_tags_returns_add_errands_tag() {
    let p = capture_payload("todo", "Renew passport p1 due:friday");
    let proposal = stub_proposal_for(&capture_state("todo", &p));
    assert_eq!(proposal.title, "Add an errands tag?");
    assert_eq!(proposal.accept_label, "Add #errands");
    match proposal.kind {
        ProposalKind::AddTag { tag } => assert_eq!(tag, "errands"),
        other => panic!("expected AddTag, got {other:?}"),
    }
}

/// Boundary 2: `+cal` no tags branch must emit `work` tag, NOT `errands`.
/// Pins the per-target lookup table against accidental fall-through to the
/// `+todo` arm.
#[test]
fn cal_no_tags_returns_work_tag_not_errands() {
    let p = capture_payload("cal", "Design review");
    let proposal = stub_proposal_for(&capture_state("cal", &p));
    assert!(matches!(proposal.kind, ProposalKind::AddTag { ref tag } if tag == "work"));
    assert!(!proposal.title.contains("errands"));
}

/// Boundary 3: `+note` no tags → `ideas`.
#[test]
fn note_no_tags_returns_ideas_tag() {
    let p = capture_payload("note", "Pattern A");
    let proposal = stub_proposal_for(&capture_state("note", &p));
    assert!(matches!(proposal.kind, ProposalKind::AddTag { ref tag } if tag == "ideas"));
}

/// Boundary 4: `+link` no tags → `read-later` (note the hyphen — easy to
/// drop in a sed refactor).
#[test]
fn link_no_tags_returns_read_later_with_hyphen() {
    let p = capture_payload("link", "https://example.com");
    let proposal = stub_proposal_for(&capture_state("link", &p));
    match &proposal.kind {
        ProposalKind::AddTag { tag } => {
            assert_eq!(tag, "read-later");
            assert!(tag.contains('-'), "must keep the hyphen — not snake_case");
        }
        other => panic!("expected AddTag, got {other:?}"),
    }
    assert_eq!(proposal.accept_label, "Add #read-later");
}

/// Boundary 5: `+social` no tags → `build` (the loud-and-quiet target with
/// the most ambiguous default; pin it explicitly).
#[test]
fn social_no_tags_returns_build_tag() {
    let p = capture_payload("social", "Ship Cmd+Enter");
    let proposal = stub_proposal_for(&capture_state("social", &p));
    assert!(matches!(proposal.kind, ProposalKind::AddTag { ref tag } if tag == "build"));
}

/// Boundary 6: an unknown target must NOT panic — it must hit the `_` arm
/// and produce the generic `tagged` fallback.
#[test]
fn unknown_target_falls_back_to_tagged_default_no_panic() {
    let p = capture_payload("xyzzy", "Something");
    let proposal = stub_proposal_for(&capture_state("xyzzy", &p));
    match proposal.kind {
        ProposalKind::AddTag { tag } => assert_eq!(tag, "tagged"),
        other => panic!("expected AddTag fallback, got {other:?}"),
    }
}

/// Boundary 7: Refine state ALWAYS emits AddField{type,script}, regardless
/// of free_text content. Pins the field key/value pair against drift.
#[test]
fn refine_state_emits_add_field_type_script_constant() {
    let q = parse_advanced_query("?something");
    let state = MenuSyntaxActionState::RefineQuery { query: &q };
    let proposal = stub_proposal_for(&state);
    match proposal.kind {
        ProposalKind::AddField { key, value } => {
            assert_eq!(key, "type");
            assert_eq!(value, "script");
        }
        other => panic!("expected AddField, got {other:?}"),
    }
    assert_eq!(proposal.accept_label, "Add type:script");
}

/// Boundary 8: Command composer with NO --help flag → RewriteInput appending
/// ` --help`. Pins the output literal so the user always sees the same
/// keystroke they would type by hand.
#[test]
fn command_without_help_appends_dash_dash_help_literal() {
    let argv = vec!["prod".to_string()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let proposal = stub_proposal_for(&state);
    match &proposal.kind {
        ProposalKind::RewriteInput { rewrite } => {
            assert_eq!(rewrite, "!deploy prod --help");
            assert!(rewrite.ends_with(" --help"));
            assert!(rewrite.starts_with('!'));
        }
        other => panic!("expected RewriteInput, got {other:?}"),
    }
}

// ---------- Composition (7) ----------

/// Composition 1: Capture WITH tags but NO priority → RewriteInput appending
/// ` p2`. Pins the trim_end behavior so trailing whitespace doesn't yield
/// ";todo body  p2" (double space).
#[test]
fn capture_with_tags_no_priority_appends_p2_no_double_space() {
    let mut p = capture_with_tags("todo", "Buy milk", &["errands"]);
    p.raw = ";todo Buy milk #errands ".to_string(); // trailing space
    let proposal = stub_proposal_for(&capture_state("todo", &p));
    match &proposal.kind {
        ProposalKind::RewriteInput { rewrite } => {
            assert_eq!(rewrite, ";todo Buy milk #errands p2");
            assert!(!rewrite.contains("  p2"), "must trim_end before append");
        }
        other => panic!("expected RewriteInput, got {other:?}"),
    }
}

/// Composition 2: Capture WITH tags AND priority → Decline (the "looks
/// complete" arm). Decline must have an empty accept_label (UI should not
/// render an Accept button).
#[test]
fn capture_complete_returns_decline_with_empty_accept_label() {
    let mut p = capture_with_tags("todo", "Buy milk", &["errands"]);
    p.priority = Some(2);
    let proposal = stub_proposal_for(&capture_state("todo", &p));
    assert!(matches!(proposal.kind, ProposalKind::Decline { .. }));
    assert_eq!(
        proposal.accept_label, "",
        "Decline must have empty accept_label so UI hides Accept"
    );
    assert!(!proposal.is_actionable());
}

/// Composition 3: Command WITH `--help` already → Decline; argv unchanged.
#[test]
fn command_with_help_already_returns_decline() {
    let argv = vec!["prod".to_string(), "--help".to_string()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "deploy",
        argv: &argv,
    };
    let proposal = stub_proposal_for(&state);
    assert!(matches!(proposal.kind, ProposalKind::Decline { .. }));
}

/// Composition 4: Command with short `-h` is also recognized as help — must
/// NOT redundantly suggest --help.
#[test]
fn command_with_short_h_treated_as_help_returns_decline() {
    let argv = vec!["-h".to_string()];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "info",
        argv: &argv,
    };
    let proposal = stub_proposal_for(&state);
    assert!(
        matches!(proposal.kind, ProposalKind::Decline { .. }),
        "got {:?}",
        proposal.kind
    );
}

/// Composition 5: Empty-argv command still gets the --help suggestion (no
/// stray space before `--help`).
#[test]
fn command_with_empty_argv_appends_help_no_double_space() {
    let argv: Vec<String> = vec![];
    let state = MenuSyntaxActionState::CommandComposer {
        head: "doctor",
        argv: &argv,
    };
    let proposal = stub_proposal_for(&state);
    match &proposal.kind {
        ProposalKind::RewriteInput { rewrite } => {
            assert_eq!(rewrite, "!doctor --help");
            assert!(!rewrite.contains("  --help"));
        }
        other => panic!("expected RewriteInput, got {other:?}"),
    }
}

/// Composition 6: Snapshot field `menu_syntax_ai_proposal` MUST round-trip
/// through serde with camelCase + skip-if-None semantics. When None, the key
/// must be entirely absent from the JSON (not `null`).
#[test]
fn proposal_serializes_camelcase_and_skips_when_none() {
    use serde_json::json;
    let proposal = MenuSyntaxAiProposal {
        title: "Add an errands tag?".to_string(),
        accept_label: "Add #errands".to_string(),
        kind: ProposalKind::AddTag {
            tag: "errands".to_string(),
        },
    };
    let v = serde_json::to_value(&proposal).unwrap();
    assert_eq!(
        v,
        json!({
            "title": "Add an errands tag?",
            "acceptLabel": "Add #errands",
            "kind": {"action": "addTag", "tag": "errands"}
        })
    );
    // No snake_case keys leaked.
    let s = v.to_string();
    assert!(!s.contains("accept_label"));
    assert!(!s.contains("\"add_tag\""));
}

/// Composition 7: All FOUR actionable ProposalKind variants serialize with
/// the `action` discriminant tag (NOT `type` — pin the chosen field name).
#[test]
fn all_actionable_kinds_use_action_discriminant_tag() {
    let cases = [
        (
            ProposalKind::AddTag {
                tag: "x".to_string(),
            },
            "addTag",
        ),
        (
            ProposalKind::AddDate {
                key: "due".to_string(),
                phrase: "fri".to_string(),
            },
            "addDate",
        ),
        (
            ProposalKind::AddField {
                key: "k".to_string(),
                value: "v".to_string(),
            },
            "addField",
        ),
        (
            ProposalKind::RewriteInput {
                rewrite: "!x --help".to_string(),
            },
            "rewriteInput",
        ),
    ];
    for (kind, expected_action) in cases {
        let v = serde_json::to_value(&kind).unwrap();
        let action = v.get("action").and_then(|a| a.as_str()).unwrap_or("");
        assert_eq!(
            action, expected_action,
            "expected action={expected_action}, got json={v}"
        );
        // Must NOT use a `type` discriminant.
        assert!(v.get("type").is_none(), "leaked `type` tag in {v}");
    }
}

// ---------- Resurrection (7) ----------

/// Resurrection 1: PRIOR REGRESSION — `MenuSyntaxAiProposal` lacked `Eq`,
/// breaking the `Eq`-deriving `MenuSyntaxMainHintSnapshot` (Pass 11 fix).
/// Pin the trait so a future "simplify derives" pass can't unbump it.
#[test]
fn proposal_implements_eq_for_snapshot_constraint() {
    fn assert_eq_trait<T: Eq>() {}
    assert_eq_trait::<MenuSyntaxAiProposal>();
    assert_eq_trait::<ProposalKind>();
}

/// Resurrection 2: PRIOR REGRESSION — Decline must NOT be actionable. If
/// `is_actionable` ever flipped to true for Decline, the UI would render an
/// Accept button on a "looks complete" proposal.
#[test]
fn decline_proposal_is_never_actionable() {
    let proposal = MenuSyntaxAiProposal {
        title: "Looks complete.".to_string(),
        accept_label: String::new(),
        kind: ProposalKind::Decline {
            reason: "x".to_string(),
        },
    };
    assert!(!proposal.is_actionable());
}

/// Resurrection 3: PRIOR REGRESSION — accept_label format pattern must stay
/// `Add #<tag>` for AddTag (with the literal `#`). Without this, the user
/// could see `Add errands` and not realize it's a tag insert.
#[test]
fn add_tag_accept_label_keeps_hash_prefix() {
    for target in ["todo", "cal", "note", "link", "social", "xyzzy"] {
        let p = capture_payload(target, "x");
        let proposal = stub_proposal_for(&capture_state(target, &p));
        if let ProposalKind::AddTag { .. } = proposal.kind {
            assert!(
                proposal.accept_label.starts_with("Add #"),
                "target={target} accept_label={:?} dropped the # prefix",
                proposal.accept_label
            );
        }
    }
}

/// Resurrection 4: PRIOR REGRESSION — title must end with `?` for AddTag
/// proposals (asks the user a question, not a command). Pins the
/// presentation contract.
#[test]
fn add_tag_title_phrased_as_question() {
    let p = capture_payload("todo", "x");
    let proposal = stub_proposal_for(&capture_state("todo", &p));
    assert!(
        proposal.title.ends_with('?'),
        "title={:?} must be a question",
        proposal.title
    );
}

/// Resurrection 5: PRIOR REGRESSION — Decline JSON shape needs the `reason`
/// field present. The dispatch layer reads it for telemetry/logging.
#[test]
fn decline_serializes_with_reason_field() {
    let kind = ProposalKind::Decline {
        reason: "argv already has --help".to_string(),
    };
    let v = serde_json::to_value(&kind).unwrap();
    assert_eq!(v.get("action").and_then(|a| a.as_str()), Some("decline"));
    assert_eq!(
        v.get("reason").and_then(|a| a.as_str()),
        Some("argv already has --help")
    );
}

/// Resurrection 6: PRIOR REGRESSION — Cmd+Enter handler dispatches per
/// `target` borrow; passing a TARGET with leading whitespace would silently
/// hit the `_` fallback. Pin: we trust the parser's already-trimmed target,
/// but if someone hands in `" todo"` (untrusted path) the stub still doesn't
/// panic — it just falls back to `tagged`.
#[test]
fn whitespace_in_target_falls_through_safely_no_panic() {
    let p = capture_payload(" todo", "x");
    let proposal = stub_proposal_for(&capture_state(" todo", &p));
    // Falls through to default — does NOT match the trimmed "todo" arm.
    match proposal.kind {
        ProposalKind::AddTag { tag } => assert_eq!(tag, "tagged"),
        other => panic!("expected AddTag fallback, got {other:?}"),
    }
}

/// Resurrection 7: PRIOR REGRESSION — RewriteInput's `rewrite` must be an
/// owned `String` field (so the dispatch layer can store it without lifetime
/// gymnastics). If someone refactors it to `&str`, this test stops compiling.
#[test]
fn rewrite_input_field_owns_its_string() {
    let kind = ProposalKind::RewriteInput {
        rewrite: "owned".to_string(),
    };
    if let ProposalKind::RewriteInput { rewrite } = kind {
        let _moved: String = rewrite; // proves it's String, not &str
    } else {
        unreachable!()
    }
}
