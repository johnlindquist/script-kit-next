//! Run 11 Pass 36 — attacker probe of [[src/app_impl/menu_syntax_ai_apply.rs#apply_proposal]]
//! (added Pass 35). Pure-function attacker — no UI surface.
//!
//! Categories: Boundary, Composition, Resurrection. Actions: 22.
//!
//! Mirror motivation: Pass 32 found `apply_safe_effect`'s DefaultTime arm
//! does `format!("{trimmed} start:\"{phrase}\"")` with no escaping. The
//! new applier has the same shape in its AddDate arm
//! (`format!("{key}:\"{phrase}\"")`) and AddField arm
//! (`format!("{key}={value}")`). This pass attacks both.

use script_kit_gpui::menu_syntax_ai::{MenuSyntaxAiProposal, ProposalKind};
use script_kit_gpui::menu_syntax_ai_apply::{apply_proposal, ProposalApplyAction, ProposalEffect};

fn add_tag(tag: &str) -> MenuSyntaxAiProposal {
    MenuSyntaxAiProposal {
        title: "t".to_string(),
        accept_label: "a".to_string(),
        kind: ProposalKind::AddTag {
            tag: tag.to_string(),
        },
    }
}

fn add_date(key: &str, phrase: &str) -> MenuSyntaxAiProposal {
    MenuSyntaxAiProposal {
        title: "t".to_string(),
        accept_label: "a".to_string(),
        kind: ProposalKind::AddDate {
            key: key.to_string(),
            phrase: phrase.to_string(),
        },
    }
}

fn add_field(key: &str, value: &str) -> MenuSyntaxAiProposal {
    MenuSyntaxAiProposal {
        title: "t".to_string(),
        accept_label: "a".to_string(),
        kind: ProposalKind::AddField {
            key: key.to_string(),
            value: value.to_string(),
        },
    }
}

fn rewrite(text: &str) -> MenuSyntaxAiProposal {
    MenuSyntaxAiProposal {
        title: "t".to_string(),
        accept_label: "a".to_string(),
        kind: ProposalKind::RewriteInput {
            rewrite: text.to_string(),
        },
    }
}

fn decline() -> MenuSyntaxAiProposal {
    MenuSyntaxAiProposal {
        title: "t".to_string(),
        accept_label: "a".to_string(),
        kind: ProposalKind::Decline {
            reason: "r".to_string(),
        },
    }
}

// ============================================================================
// BOUNDARY (8 actions)
// ============================================================================

#[test]
fn boundary_01_empty_tag_appends_bare_hash_no_letters() {
    // Current behavior: `format!("#{tag}")` with empty tag → "#". No
    // input validation rejects this. Pinned current behavior; if a
    // future caller validates non-empty tags this test flips.
    let out = apply_proposal(";todo Buy", &add_tag(""), ProposalApplyAction::Accept);
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Buy #".to_string()
        }
    );
}

#[test]
fn boundary_02_unicode_emoji_tag_passes_through() {
    let out = apply_proposal(
        ";todo Buy",
        &add_tag("\u{1F680}rocket"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Buy #\u{1F680}rocket".to_string()
        }
    );
}

#[test]
fn boundary_03_tag_containing_space_emits_unbalanced_token() {
    // `#two words` is ambiguous — downstream tokenizer would split at
    // the space. Pinned current behavior; caller-side validation could
    // reject space-bearing tags before they reach the applier.
    let out = apply_proposal(
        ";todo Buy",
        &add_tag("two words"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Buy #two words".to_string()
        }
    );
}

#[test]
fn boundary_04_tag_with_leading_hash_doubles_the_hash() {
    // Author-doubled hash: tag "#errands" → "##errands". Caller is
    // expected to pass the bare token; if not, the appender does not
    // strip a leading `#`. Pinned current behavior.
    let out = apply_proposal(
        ";todo Buy",
        &add_tag("#errands"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Buy ##errands".to_string()
        }
    );
}

#[test]
fn boundary_05_add_date_phrase_with_quote_is_backslash_escaped_PINNED() {
    // Run 11 Pass #41 (Fix): the original Pass-36 [?] is closed. The
    // AddDate arm now routes the phrase through `quote_for_filter_value`
    // which backslash-escapes `"` and `\`. Phrases containing `"` produce
    // properly-escaped, balanced output.
    let out = apply_proposal(
        ";todo Renew",
        &add_date("due", "today \"9am\""),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Renew due:\"today \\\"9am\\\"\"".to_string()
        }
    );
    // Falsifier guard: count UNESCAPED `"` chars — must be exactly 2
    // (the opening and closing wrappers around the phrase).
    if let ProposalEffect::SetFilterText { new_text } = out {
        let mut unescaped = 0;
        let mut prev_backslash = false;
        for c in new_text.chars() {
            if c == '"' && !prev_backslash {
                unescaped += 1;
            }
            prev_backslash = c == '\\' && !prev_backslash;
        }
        assert_eq!(unescaped, 2, "balanced wrappers expected; got {new_text:?}");
    }
}

#[test]
fn boundary_06_add_date_phrase_with_backslash_is_doubled_PINNED() {
    // Run 11 Pass #41 (Fix): backslashes are now escaped FIRST (before
    // quote escaping) by `quote_for_filter_value`, so a literal `\` in
    // the phrase becomes `\\` in the output. A trailing `\` no longer
    // confuses downstream parsers as an escape-the-closing-quote.
    let out = apply_proposal(
        ";todo Renew",
        &add_date("due", "today \\"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Renew due:\"today \\\\\"".to_string()
        }
    );
}

#[test]
fn boundary_07_rewrite_with_empty_string_replaces_input_with_empty() {
    // Wholesale-replace: empty rewrite replaces input with empty
    // string. The model owns the entire output, even when empty.
    let out = apply_proposal("anything", &rewrite(""), ProposalApplyAction::Accept);
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: "".to_string()
        }
    );
}

#[test]
fn boundary_08_add_field_with_empty_value_emits_bare_equals() {
    // `key=` is a valid token in the grammar (means key with empty
    // value). Pinned current behavior.
    let out = apply_proposal(
        ";expense Lunch",
        &add_field("amount", ""),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";expense Lunch amount=".to_string()
        }
    );
}

// ============================================================================
// COMPOSITION (8 actions)
// ============================================================================

#[test]
fn composition_09_chained_accepts_compose_left_to_right() {
    // Real-world flow: user accepts proposal A, then a new proposal B
    // is offered against the new input. Verify the applier is
    // composable — feeding the output of one Accept into another
    // produces the expected concatenation.
    let p1 = add_tag("errands");
    let p2 = add_field("amount", "18.50");
    let after_first = match apply_proposal(";todo Buy milk", &p1, ProposalApplyAction::Accept) {
        ProposalEffect::SetFilterText { new_text } => new_text,
        ProposalEffect::Dismiss => panic!("expected SetFilterText"),
    };
    let after_second = apply_proposal(&after_first, &p2, ProposalApplyAction::Accept);
    assert_eq!(
        after_second,
        ProposalEffect::SetFilterText {
            new_text: ";todo Buy milk #errands amount=18.50".to_string()
        }
    );
}

#[test]
fn composition_10_accept_then_dismiss_dismiss_wins() {
    // Sanity: Dismiss on the same proposal after a hypothetical Accept
    // still yields Dismiss. Applier is stateless — each call is fresh.
    let p = add_tag("errands");
    let _accepted = apply_proposal(";todo Buy", &p, ProposalApplyAction::Accept);
    let dismissed = apply_proposal(";todo Buy", &p, ProposalApplyAction::Dismiss);
    assert_eq!(dismissed, ProposalEffect::Dismiss);
}

#[test]
fn composition_11_decline_with_dismiss_action_returns_dismiss() {
    // Decline + Dismiss = Dismiss (already covered in lib tests as
    // part of Dismiss-on-any-kind, but pin it here as a discrete cell
    // in the (action × kind) cross-product).
    assert_eq!(
        apply_proposal("anything", &decline(), ProposalApplyAction::Dismiss),
        ProposalEffect::Dismiss
    );
}

#[test]
fn composition_12_trim_end_handles_tab_character() {
    // trim_end is `char::is_whitespace`-based, which includes `\t`.
    // Verify a tab-separated trailing input collapses cleanly.
    let out = apply_proposal(
        ";todo Buy milk\t",
        &add_tag("errands"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Buy milk #errands".to_string()
        }
    );
}

#[test]
fn composition_13_trim_end_handles_newline() {
    // `\n` in input — also whitespace, also trimmed. Pinned.
    let out = apply_proposal(
        ";todo Buy milk\n",
        &add_tag("errands"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Buy milk #errands".to_string()
        }
    );
}

#[test]
fn composition_14_add_field_key_containing_equals_produces_double_equals() {
    // Boundary that doubles as composition: a key like `amount=base`
    // produces `amount=base=18.50`. The downstream tokenizer would
    // split on the FIRST `=` so this is structurally fine, but it
    // surfaces a sharp edge an author might trip over. Pinned current
    // behavior; not filing `[?]` because the grammar tolerates it.
    let out = apply_proposal(
        ";expense Lunch",
        &add_field("amount=base", "18.50"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";expense Lunch amount=base=18.50".to_string()
        }
    );
}

#[test]
fn composition_15_add_date_key_containing_colon_produces_double_colon() {
    // Same shape: key `due:soft` → `due:soft:"friday"`. Pinned.
    let out = apply_proposal(
        ";todo Renew",
        &add_date("due:soft", "friday"),
        ProposalApplyAction::Accept,
    );
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: ";todo Renew due:soft:\"friday\"".to_string()
        }
    );
}

#[test]
fn composition_16_rewrite_preserves_leading_whitespace() {
    // RewriteInput is wholesale — leading whitespace in the rewrite
    // string is preserved verbatim. Pinned: the model owns the
    // ENTIRE output, including whitespace. Falsifier: a future
    // change that trims rewrite would flip this.
    let p = rewrite("   +todo padded");
    let out = apply_proposal("anything", &p, ProposalApplyAction::Accept);
    assert_eq!(
        out,
        ProposalEffect::SetFilterText {
            new_text: "   +todo padded".to_string()
        }
    );
}

// ============================================================================
// RESURRECTION (6 actions)
// ============================================================================

#[test]
fn resurrection_17_same_proposal_per_input_idempotent() {
    let p = add_tag("errands");
    let inputs = [";todo a", ";todo b", ";todo c"];
    for inp in inputs {
        let out1 = apply_proposal(inp, &p, ProposalApplyAction::Accept);
        let out2 = apply_proposal(inp, &p, ProposalApplyAction::Accept);
        assert_eq!(out1, out2, "non-idempotent for input {inp:?}");
    }
}

#[test]
fn resurrection_18_clone_proposal_yields_equal_effect() {
    let p1 = add_date("due", "tomorrow");
    let p2 = p1.clone();
    let out1 = apply_proposal(";todo X", &p1, ProposalApplyAction::Accept);
    let out2 = apply_proposal(";todo X", &p2, ProposalApplyAction::Accept);
    assert_eq!(out1, out2);
}

#[test]
fn resurrection_19_clone_effect_equality() {
    let p = rewrite(">deploy --prod");
    let e1 = apply_proposal(">deploy", &p, ProposalApplyAction::Accept);
    let e2 = e1.clone();
    assert_eq!(e1, e2);
}

#[test]
fn resurrection_20_mutating_proposal_between_calls_flows_through() {
    // Different proposal instances yield different effects. Pin that
    // the applier reads the live proposal each call (no caching).
    let p1 = add_tag("a");
    let p2 = add_tag("b");
    let out1 = apply_proposal(";todo X", &p1, ProposalApplyAction::Accept);
    let out2 = apply_proposal(";todo X", &p2, ProposalApplyAction::Accept);
    assert_ne!(out1, out2);
    assert_eq!(
        out1,
        ProposalEffect::SetFilterText {
            new_text: ";todo X #a".to_string()
        }
    );
    assert_eq!(
        out2,
        ProposalEffect::SetFilterText {
            new_text: ";todo X #b".to_string()
        }
    );
}

#[test]
fn resurrection_21_pure_dispatch_alternating_accept_dismiss_same_state() {
    // Accept-Dismiss-Accept-Dismiss in same state: Accept always
    // yields SetFilterText, Dismiss always Dismiss. No state leaks.
    let p = add_tag("errands");
    for _ in 0..3 {
        assert!(matches!(
            apply_proposal(";todo X", &p, ProposalApplyAction::Accept),
            ProposalEffect::SetFilterText { .. }
        ));
        assert_eq!(
            apply_proposal(";todo X", &p, ProposalApplyAction::Dismiss),
            ProposalEffect::Dismiss
        );
    }
}

#[test]
fn resurrection_22_repeated_dismiss_on_decline_is_idempotent() {
    let p = decline();
    for _ in 0..5 {
        assert_eq!(
            apply_proposal("anything", &p, ProposalApplyAction::Accept),
            ProposalEffect::Dismiss
        );
        assert_eq!(
            apply_proposal("anything", &p, ProposalApplyAction::Dismiss),
            ProposalEffect::Dismiss
        );
    }
}
