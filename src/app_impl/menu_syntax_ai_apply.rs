//! Pure applier for [[MenuSyntaxAiProposal]] in the inline-AI hint surface.
//!
//! The inline-AI hint card asks the user to Tab/Enter to accept a proposal
//! or Esc to dismiss it. This module is the pure decision layer for that
//! interaction: given the current input text, the active proposal, and
//! which key the user pressed, return a [[ProposalEffect]] that the UI
//! glue layer applies (or ignores). It carries no GPUI state and never
//! mutates anything on its own — same shape as [[apply_safe_effect]] for
//! Cmd+K actions (Pass 29).
//!
//! Surface ships data layer for the `ai-proposal-accept-dismiss` story
//! (Run 11 Pass 35). UI wiring (key handler in the inline-AI hint
//! component) requires touching the renderer + a binary rebuild and is
//! the deferred `[~]` half.

use crate::menu_syntax_ai::{MenuSyntaxAiProposal, ProposalKind};

/// Which key the user pressed on the inline-AI hint card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalApplyAction {
    /// Tab or Enter — apply the proposal to the current input.
    Accept,
    /// Esc — dismiss the proposal without changing input.
    Dismiss,
}

/// What the UI should do with the current input after the user keyed
/// Accept/Dismiss on a proposal. The applier returns this; the UI glue
/// layer threads it back to the input field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalEffect {
    /// Replace the filter input with this exact string. Used by Accept on
    /// any actionable proposal kind (AddTag/AddDate/AddField/RewriteInput).
    SetFilterText { new_text: String },
    /// Tear down the proposal hint card. The input text is unchanged.
    /// Used by Dismiss on any proposal, and by Accept on Decline (since
    /// there is nothing to apply).
    Dismiss,
}

/// Resolve the user's keypress against the proposal.
///
/// - `Accept` on an actionable proposal → `SetFilterText`.
/// - `Accept` on a Decline proposal → `Dismiss` (nothing to apply).
/// - `Dismiss` on anything → `Dismiss` (input unchanged).
pub fn apply_proposal(
    current_input: &str,
    proposal: &MenuSyntaxAiProposal,
    action: ProposalApplyAction,
) -> ProposalEffect {
    match action {
        ProposalApplyAction::Dismiss => ProposalEffect::Dismiss,
        ProposalApplyAction::Accept => match &proposal.kind {
            ProposalKind::AddTag { tag } => append_token(current_input, &format!("#{tag}")),
            ProposalKind::AddDate { key, phrase } => append_token(
                current_input,
                &format!(
                    "{key}:{}",
                    crate::menu_syntax::quote_for_filter_value(phrase)
                ),
            ),
            ProposalKind::AddField { key, value } => {
                append_token(current_input, &format!("{key}={value}"))
            }
            ProposalKind::RewriteInput { rewrite } => ProposalEffect::SetFilterText {
                new_text: rewrite.clone(),
            },
            ProposalKind::Decline { .. } => ProposalEffect::Dismiss,
        },
    }
}

fn append_token(current_input: &str, token: &str) -> ProposalEffect {
    let trimmed = current_input.trim_end();
    let new_text = if trimmed.is_empty() {
        token.to_string()
    } else {
        format!("{trimmed} {token}")
    };
    ProposalEffect::SetFilterText { new_text }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax_ai::{MenuSyntaxAiProposal, ProposalKind};

    fn add_tag(tag: &str) -> MenuSyntaxAiProposal {
        MenuSyntaxAiProposal {
            title: format!("Add #{tag}?"),
            accept_label: format!("Add #{tag}"),
            kind: ProposalKind::AddTag {
                tag: tag.to_string(),
            },
        }
    }

    fn rewrite(text: &str) -> MenuSyntaxAiProposal {
        MenuSyntaxAiProposal {
            title: "Rewrite?".to_string(),
            accept_label: "Apply".to_string(),
            kind: ProposalKind::RewriteInput {
                rewrite: text.to_string(),
            },
        }
    }

    fn decline() -> MenuSyntaxAiProposal {
        MenuSyntaxAiProposal {
            title: "No suggestion".to_string(),
            accept_label: "Dismiss".to_string(),
            kind: ProposalKind::Decline {
                reason: "model declined".to_string(),
            },
        }
    }

    #[test]
    fn dismiss_on_any_proposal_returns_dismiss() {
        let p = add_tag("errands");
        assert_eq!(
            apply_proposal(";todo Buy milk", &p, ProposalApplyAction::Dismiss),
            ProposalEffect::Dismiss
        );
        assert_eq!(
            apply_proposal("", &decline(), ProposalApplyAction::Dismiss),
            ProposalEffect::Dismiss
        );
    }

    #[test]
    fn accept_add_tag_appends_hash_token_to_input() {
        let p = add_tag("errands");
        let out = apply_proposal(";todo Buy milk", &p, ProposalApplyAction::Accept);
        assert_eq!(
            out,
            ProposalEffect::SetFilterText {
                new_text: ";todo Buy milk #errands".to_string()
            }
        );
    }

    #[test]
    fn accept_add_tag_on_trailing_space_input_collapses_to_single_separator() {
        // Input has a trailing space already — appender must trim_end to
        // avoid a double space.
        let p = add_tag("errands");
        let out = apply_proposal(";todo Buy milk   ", &p, ProposalApplyAction::Accept);
        assert_eq!(
            out,
            ProposalEffect::SetFilterText {
                new_text: ";todo Buy milk #errands".to_string()
            }
        );
    }

    #[test]
    fn accept_add_tag_on_empty_input_yields_bare_token() {
        let p = add_tag("errands");
        assert_eq!(
            apply_proposal("", &p, ProposalApplyAction::Accept),
            ProposalEffect::SetFilterText {
                new_text: "#errands".to_string()
            }
        );
        assert_eq!(
            apply_proposal("   ", &p, ProposalApplyAction::Accept),
            ProposalEffect::SetFilterText {
                new_text: "#errands".to_string()
            }
        );
    }

    #[test]
    fn accept_add_date_appends_colon_quoted_phrase() {
        let p = MenuSyntaxAiProposal {
            title: "Add a due date?".to_string(),
            accept_label: "Add due:friday 2pm".to_string(),
            kind: ProposalKind::AddDate {
                key: "due".to_string(),
                phrase: "friday 2pm".to_string(),
            },
        };
        assert_eq!(
            apply_proposal(";todo Renew passport", &p, ProposalApplyAction::Accept),
            ProposalEffect::SetFilterText {
                new_text: ";todo Renew passport due:\"friday 2pm\"".to_string()
            }
        );
    }

    #[test]
    fn accept_add_field_appends_key_equals_value() {
        let p = MenuSyntaxAiProposal {
            title: "Add amount?".to_string(),
            accept_label: "Add amount=18.50".to_string(),
            kind: ProposalKind::AddField {
                key: "amount".to_string(),
                value: "18.50".to_string(),
            },
        };
        assert_eq!(
            apply_proposal(";expense Lunch", &p, ProposalApplyAction::Accept),
            ProposalEffect::SetFilterText {
                new_text: ";expense Lunch amount=18.50".to_string()
            }
        );
    }

    #[test]
    fn accept_rewrite_replaces_input_wholesale_per_story_receipt() {
        // Story receipt: setFilter ">deploy --" + cmd-enter + tab + getState
        // shows filterText:">deploy -- prod --dry-run".
        let p = rewrite(">deploy -- prod --dry-run");
        let out = apply_proposal(">deploy --", &p, ProposalApplyAction::Accept);
        assert_eq!(
            out,
            ProposalEffect::SetFilterText {
                new_text: ">deploy -- prod --dry-run".to_string()
            }
        );
    }

    #[test]
    fn accept_rewrite_replaces_input_even_when_unrelated() {
        // RewriteInput is wholesale — the model owns the entire output
        // string; the current input contributes nothing.
        let p = rewrite(";todo Different thing entirely");
        let out = apply_proposal("anything goes here", &p, ProposalApplyAction::Accept);
        assert_eq!(
            out,
            ProposalEffect::SetFilterText {
                new_text: ";todo Different thing entirely".to_string()
            }
        );
    }

    #[test]
    fn accept_on_decline_proposal_returns_dismiss() {
        // Decline has nothing to apply — Accept and Dismiss both tear
        // down the hint card without changing input.
        assert_eq!(
            apply_proposal(";todo X", &decline(), ProposalApplyAction::Accept),
            ProposalEffect::Dismiss
        );
    }

    #[test]
    fn dismiss_on_rewrite_does_not_replace_input_falsifier() {
        // Falsifier: a future contributor who routes Dismiss through the
        // Accept path would silently apply the rewrite. This test pins
        // that Dismiss is the correct branch — input must be untouched.
        let p = rewrite(">deploy -- prod --dry-run");
        assert_eq!(
            apply_proposal(">deploy --", &p, ProposalApplyAction::Dismiss),
            ProposalEffect::Dismiss
        );
    }

    #[test]
    fn applier_is_pure_repeat_calls_yield_identical_effects() {
        let p = add_tag("errands");
        let out1 = apply_proposal(";todo Buy", &p, ProposalApplyAction::Accept);
        let out2 = apply_proposal(";todo Buy", &p, ProposalApplyAction::Accept);
        let out3 = apply_proposal(";todo Buy", &p, ProposalApplyAction::Accept);
        assert_eq!(out1, out2);
        assert_eq!(out2, out3);
    }
}
