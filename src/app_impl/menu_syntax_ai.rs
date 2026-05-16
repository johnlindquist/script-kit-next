// doc-anchor-removed: [[removed-docs Syntax#Cmd+Enter Inline AI Proposal]]
//
// Pure adapter that flattens [[src/menu_syntax/ai.rs#MenuSyntaxAiResponse]]
// into the snapshot the inline-AI hint surface renders. Cmd+Enter in
// capture/refine/command states triggers an AI request; the response is
// mapped to a `MenuSyntaxAiProposal` and stuffed into
// `MenuSyntaxMainHintSnapshot.menu_syntax_ai_proposal` so the renderer can
// show the title + accept-label without re-running mapping rules.
//
// The runtime Cmd+Enter handler + ACP/LLM call wiring is the deferred UI
// integration; this module is the pure data layer per Pass 5
// `menu-syntax-ai-contract` precedent.

use serde::{Deserialize, Serialize};

use crate::menu_syntax::ai::MenuSyntaxAiResponse;

/// One AI proposal renderable in the hint surface. Mirrors the Pass-5
/// `MenuSyntaxAiResponse` enum with all four actionable kinds collapsed
/// into a unified shape carrying `title` + `accept_label` (the user-facing
/// strings the hint card shows) plus a structured `kind` payload the
/// accept-handler reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxAiProposal {
    pub title: String,
    pub accept_label: String,
    pub kind: ProposalKind,
}

/// The input identity that produced a pending proposal. Runtime surfaces use
/// this to prevent a proposal generated for one filter from being applied to a
/// later filter.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MenuSyntaxAiProposalOrigin {
    pub raw_input: String,
    pub target: Option<String>,
}

/// Runtime-owned proposal plus the input identity that produced it.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingMenuSyntaxAiProposal {
    pub origin: MenuSyntaxAiProposalOrigin,
    pub proposal: MenuSyntaxAiProposal,
}

#[allow(dead_code)]
impl PendingMenuSyntaxAiProposal {
    pub(crate) fn new(
        raw_input: String,
        target: Option<String>,
        proposal: MenuSyntaxAiProposal,
    ) -> Self {
        Self {
            origin: MenuSyntaxAiProposalOrigin { raw_input, target },
            proposal,
        }
    }

    pub(crate) fn is_current_for(&self, current_input: &str) -> bool {
        self.origin.raw_input == current_input
    }
}

/// What the accept-handler should do when the user presses Tab/Enter on the
/// proposal. Pure data — no GPUI types — so the spec layer is testable
/// without a window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum ProposalKind {
    AddTag {
        tag: String,
    },
    AddDate {
        key: String,
        phrase: String,
    },
    AddField {
        key: String,
        value: String,
    },
    RewriteInput {
        rewrite: String,
    },
    /// Model declined to suggest. The hint card shows the reason but the
    /// accept-handler does nothing.
    Decline {
        reason: String,
    },
}

impl MenuSyntaxAiProposal {
    /// True when accepting the proposal would change the launcher input.
    /// `Decline` proposals are not actionable; `RewriteInput` and the three
    /// inline-token kinds all are.
    pub fn is_actionable(&self) -> bool {
        !matches!(self.kind, ProposalKind::Decline { .. })
    }
}

/// Map an AI response into a renderable proposal. The Pass-5 response
/// enum already carries `title` and `accept_label` for every variant, so
/// this is a flat structural translation rather than a content rewrite.
pub fn proposal_from_response(response: &MenuSyntaxAiResponse) -> MenuSyntaxAiProposal {
    match response {
        MenuSyntaxAiResponse::AddTag {
            tag,
            title,
            accept_label,
        } => MenuSyntaxAiProposal {
            title: title.clone(),
            accept_label: accept_label.clone(),
            kind: ProposalKind::AddTag { tag: tag.clone() },
        },
        MenuSyntaxAiResponse::AddDate {
            key,
            phrase,
            title,
            accept_label,
        } => MenuSyntaxAiProposal {
            title: title.clone(),
            accept_label: accept_label.clone(),
            kind: ProposalKind::AddDate {
                key: key.clone(),
                phrase: phrase.clone(),
            },
        },
        MenuSyntaxAiResponse::AddField {
            key,
            value,
            title,
            accept_label,
        } => MenuSyntaxAiProposal {
            title: title.clone(),
            accept_label: accept_label.clone(),
            kind: ProposalKind::AddField {
                key: key.clone(),
                value: value.clone(),
            },
        },
        MenuSyntaxAiResponse::RewriteInput {
            rewrite,
            title,
            accept_label,
        } => MenuSyntaxAiProposal {
            title: title.clone(),
            accept_label: accept_label.clone(),
            kind: ProposalKind::RewriteInput {
                rewrite: rewrite.clone(),
            },
        },
        MenuSyntaxAiResponse::NoSuggestion { reason } => MenuSyntaxAiProposal {
            // The hint card uses `title` for both decline and acceptance, so
            // surface the reason there with a neutral `accept_label`.
            title: reason.clone(),
            accept_label: String::new(),
            kind: ProposalKind::Decline {
                reason: reason.clone(),
            },
        },
    }
}

/// Run 12 Pass 11 — deterministic stub proposal generator. Used by the
/// Cmd+Enter handler so the receipt is observable without an LLM round-trip.
/// Replace with the real ACP/LLM call when that wiring lands; the function
/// signature is stable.
///
/// Heuristics:
/// - Capture composer w/ no tags: suggest `AddTag { tag: "errands" }` for
///   `+todo`, `tag: "work"` for `+cal`, `tag: "ideas"` for `+note`,
///   `tag: "read-later"` for `+link`. Title pattern: `"Add an <tag> tag?"`.
/// - Capture composer with tags already: suggest `RewriteInput` that adds
///   `p2` priority if missing.
/// - Refine: suggest `AddField { key: "type", value: "script" }` if missing.
/// - Command composer: suggest `RewriteInput` that adds `--help` flag.
pub fn stub_proposal_for(
    state: &crate::menu_syntax::MenuSyntaxActionState<'_>,
) -> MenuSyntaxAiProposal {
    use crate::menu_syntax::MenuSyntaxActionState;
    match state {
        MenuSyntaxActionState::CaptureComposer {
            target, payload, ..
        } => {
            let suggested_tag = match *target {
                "todo" => "errands",
                "cal" => "work",
                "note" => "ideas",
                "link" => "read-later",
                "social" => "build",
                _ => "tagged",
            };
            if payload.tags.is_empty() {
                MenuSyntaxAiProposal {
                    title: format!("Add an {suggested_tag} tag?"),
                    accept_label: format!("Add #{suggested_tag}"),
                    kind: ProposalKind::AddTag {
                        tag: suggested_tag.to_string(),
                    },
                }
            } else if payload.priority.is_none() {
                let new_text = format!("{} p2", payload.raw.trim_end());
                MenuSyntaxAiProposal {
                    title: "Add a default priority?".to_string(),
                    accept_label: "Add p2".to_string(),
                    kind: ProposalKind::RewriteInput { rewrite: new_text },
                }
            } else {
                MenuSyntaxAiProposal {
                    title: "Looks complete.".to_string(),
                    accept_label: String::new(),
                    kind: ProposalKind::Decline {
                        reason: "input already has tags and priority".to_string(),
                    },
                }
            }
        }
        MenuSyntaxActionState::RefineQuery { .. } => MenuSyntaxAiProposal {
            title: "Filter to scripts only?".to_string(),
            accept_label: "Add type:script".to_string(),
            kind: ProposalKind::AddField {
                key: "type".to_string(),
                value: "script".to_string(),
            },
        },
        MenuSyntaxActionState::CommandComposer { head, argv } => {
            let needs_help = !argv.iter().any(|a| a == "--help" || a == "-h");
            if needs_help {
                let mut new_text = format!("!{head}");
                for a in argv.iter() {
                    new_text.push(' ');
                    new_text.push_str(a);
                }
                new_text.push_str(" --help");
                MenuSyntaxAiProposal {
                    title: "Add --help to inspect args?".to_string(),
                    accept_label: "Add --help".to_string(),
                    kind: ProposalKind::RewriteInput { rewrite: new_text },
                }
            } else {
                MenuSyntaxAiProposal {
                    title: "Looks complete.".to_string(),
                    accept_label: String::new(),
                    kind: ProposalKind::Decline {
                        reason: "argv already has --help".to_string(),
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_tag_response_matches_story_receipt_example() {
        // Story receipt: setFilter ";todo Renew passport p1 due:friday"
        // + cmd-enter + getState shows
        // menuSyntaxAiProposal:{title:"Add an errands tag?",acceptLabel:"Add #errands"}.
        // The pure mapper produces exactly that shape from the AI response.
        let response = MenuSyntaxAiResponse::AddTag {
            tag: "errands".to_string(),
            title: "Add an errands tag?".to_string(),
            accept_label: "Add #errands".to_string(),
        };
        let proposal = proposal_from_response(&response);
        assert_eq!(proposal.title, "Add an errands tag?");
        assert_eq!(proposal.accept_label, "Add #errands");
        assert_eq!(
            proposal.kind,
            ProposalKind::AddTag {
                tag: "errands".to_string()
            }
        );
        assert!(proposal.is_actionable());
    }

    #[test]
    fn add_date_response_maps_to_proposal() {
        let response = MenuSyntaxAiResponse::AddDate {
            key: "due".to_string(),
            phrase: "friday".to_string(),
            title: "Add a due date?".to_string(),
            accept_label: "Add due:friday".to_string(),
        };
        let proposal = proposal_from_response(&response);
        assert_eq!(
            proposal.kind,
            ProposalKind::AddDate {
                key: "due".to_string(),
                phrase: "friday".to_string()
            }
        );
        assert!(proposal.is_actionable());
    }

    #[test]
    fn add_field_response_maps_to_proposal() {
        let response = MenuSyntaxAiResponse::AddField {
            key: "amount".to_string(),
            value: "12.50".to_string(),
            title: "Add an amount?".to_string(),
            accept_label: "Add amount=12.50".to_string(),
        };
        let proposal = proposal_from_response(&response);
        assert_eq!(
            proposal.kind,
            ProposalKind::AddField {
                key: "amount".to_string(),
                value: "12.50".to_string()
            }
        );
    }

    #[test]
    fn rewrite_input_response_maps_to_proposal() {
        let response = MenuSyntaxAiResponse::RewriteInput {
            rewrite: ">deploy -- prod --dry-run".to_string(),
            title: "Add safety flags?".to_string(),
            accept_label: "Use suggested argv".to_string(),
        };
        let proposal = proposal_from_response(&response);
        assert_eq!(
            proposal.kind,
            ProposalKind::RewriteInput {
                rewrite: ">deploy -- prod --dry-run".to_string(),
            }
        );
        assert!(proposal.is_actionable());
    }

    #[test]
    fn no_suggestion_response_maps_to_decline_not_actionable() {
        let response = MenuSyntaxAiResponse::NoSuggestion {
            reason: "Not enough context to refine".to_string(),
        };
        let proposal = proposal_from_response(&response);
        assert_eq!(proposal.title, "Not enough context to refine");
        assert!(proposal.accept_label.is_empty());
        assert_eq!(
            proposal.kind,
            ProposalKind::Decline {
                reason: "Not enough context to refine".to_string()
            }
        );
        assert!(!proposal.is_actionable());
    }

    #[test]
    fn proposal_serializes_camel_case_for_state_receipt() {
        // Renderable in getState — fields must be camelCase to match the
        // story receipt (acceptLabel, not accept_label).
        let proposal = MenuSyntaxAiProposal {
            title: "Add an errands tag?".to_string(),
            accept_label: "Add #errands".to_string(),
            kind: ProposalKind::AddTag {
                tag: "errands".to_string(),
            },
        };
        let json = serde_json::to_value(&proposal).expect("serialize");
        assert_eq!(json["title"], "Add an errands tag?");
        assert_eq!(json["acceptLabel"], "Add #errands");
        assert_eq!(json["kind"]["action"], "addTag");
        assert_eq!(json["kind"]["tag"], "errands");
    }

    #[test]
    fn decline_proposal_serializes_with_action_decline() {
        let proposal = proposal_from_response(&MenuSyntaxAiResponse::NoSuggestion {
            reason: "ambiguous".to_string(),
        });
        let json = serde_json::to_value(&proposal).expect("serialize");
        assert_eq!(json["kind"]["action"], "decline");
        assert_eq!(json["kind"]["reason"], "ambiguous");
        assert_eq!(json["acceptLabel"], "");
    }

    #[test]
    fn proposal_round_trips_through_json() {
        let original = MenuSyntaxAiProposal {
            title: "Add a due date?".to_string(),
            accept_label: "Add due:friday".to_string(),
            kind: ProposalKind::AddDate {
                key: "due".to_string(),
                phrase: "friday".to_string(),
            },
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: MenuSyntaxAiProposal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, parsed);
    }
}
