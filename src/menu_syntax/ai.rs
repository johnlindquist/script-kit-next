use serde::{Deserialize, Serialize};

use crate::menu_syntax::capture_schema::FieldRequirement;

/// Structured context sent to the AI model when the user presses Cmd+Enter
/// while composing a Power Syntax expression. Pure data — the inline AI
/// proposal layer (NEW src/app_impl/menu_syntax_ai.rs, lands in a follow-up
/// story) builds one from the live parse + validation result and serializes
/// it into the model prompt. Designed so the prompt template is the same
/// regardless of state, with the per-state payload supplying what the model
/// needs to suggest a useful edit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxAiRequest {
    pub raw_input: String,
    pub state: MenuSyntaxAiState,
}

/// Discriminated payload — only one variant is populated per request, matching
/// the user's current Power Syntax surface. Field names mirror the live parse
/// types so the model sees the same shapes the rest of the codebase reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MenuSyntaxAiState {
    #[serde(rename_all = "camelCase")]
    Capture {
        target: String,
        body: String,
        tags: Vec<String>,
        priority: Option<u8>,
        url: Option<String>,
        duration: Option<String>,
        kv: Vec<(String, String)>,
        date_phrases: Vec<DatePhraseHint>,
        missing_required: Vec<String>,
        recent_tags: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    Refine {
        free_text: String,
        predicates: Vec<String>,
        result_count_hint: Option<usize>,
    },
    #[serde(rename_all = "camelCase")]
    Command {
        head: String,
        fields: Vec<(String, String)>,
        argv: Vec<String>,
        recent_argv: Vec<String>,
    },
}

/// A flattened date phrase descriptor — role plus source string. Keeps the AI
/// payload schema stable even as the underlying ResolvedDate / ParsedDate
/// types evolve in W2 follow-up stories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatePhraseHint {
    pub role: String,
    pub source: String,
}

/// Convert a missing FieldRequirement Vec into the human labels the AI prompt
/// uses ("body", "date", "url", etc.). Centralized here so both the request
/// builder and the inline UI share the same label vocabulary.
pub fn missing_required_labels(missing: &[FieldRequirement]) -> Vec<String> {
    missing.iter().map(FieldRequirement::label).collect()
}

/// Build a Capture request with the new common fields. Caller supplies the
/// recent_tags set (sourced from previously-saved capture artifacts) and the
/// missing-required labels (typically `missing_required_labels(schema.missing_required(payload))`).
#[allow(clippy::too_many_arguments)]
pub fn capture_request(
    raw_input: impl Into<String>,
    target: impl Into<String>,
    body: impl Into<String>,
    tags: Vec<String>,
    priority: Option<u8>,
    url: Option<String>,
    duration: Option<String>,
    kv: Vec<(String, String)>,
    date_phrases: Vec<DatePhraseHint>,
    missing_required: Vec<String>,
    recent_tags: Vec<String>,
) -> MenuSyntaxAiRequest {
    MenuSyntaxAiRequest {
        raw_input: raw_input.into(),
        state: MenuSyntaxAiState::Capture {
            target: target.into(),
            body: body.into(),
            tags,
            priority,
            url,
            duration,
            kv,
            date_phrases,
            missing_required,
            recent_tags,
        },
    }
}

/// Structured proposal the model returns. The inline AI hint render layer
/// applies the proposal to the launcher input on Tab/Enter; Esc dismisses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MenuSyntaxAiResponse {
    /// Insert an inline `#tag` token into the input.
    #[serde(rename_all = "camelCase")]
    AddTag {
        tag: String,
        title: String,
        accept_label: String,
    },
    /// Insert a `start:"…"` (or other date-key) token.
    #[serde(rename_all = "camelCase")]
    AddDate {
        key: String,
        phrase: String,
        title: String,
        accept_label: String,
    },
    /// Insert an arbitrary `key=value` token.
    #[serde(rename_all = "camelCase")]
    AddField {
        key: String,
        value: String,
        title: String,
        accept_label: String,
    },
    /// Replace the current input with the rewritten version.
    #[serde(rename_all = "camelCase")]
    RewriteInput {
        rewrite: String,
        title: String,
        accept_label: String,
    },
    /// Model declined to suggest. The UI shows the reason but no accept action.
    NoSuggestion { reason: String },
}

/// Parse a model response. Wraps serde with a typed error so the inline AI
/// layer can distinguish "model returned junk" from "transport failure".
#[derive(Debug, Clone, PartialEq)]
pub enum AiParseError {
    EmptyInput,
    InvalidJson(String),
}

pub fn parse_response(json: &str) -> Result<MenuSyntaxAiResponse, AiParseError> {
    let trimmed = json.trim();
    if trimmed.is_empty() {
        return Err(AiParseError::EmptyInput);
    }
    serde_json::from_str(trimmed).map_err(|e| AiParseError::InvalidJson(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_request_serializes_full_shape() {
        let req = capture_request(
            ";todo Renew passport p1 due:friday",
            "todo",
            "Renew passport",
            vec![],
            Some(1),
            None,
            None,
            vec![],
            vec![DatePhraseHint {
                role: "due".into(),
                source: "friday".into(),
            }],
            vec!["body".into()],
            vec!["errands".into(), "client/acme".into()],
        );
        let json = serde_json::to_value(&req).expect("serde");
        assert_eq!(json["rawInput"], ";todo Renew passport p1 due:friday");
        assert_eq!(json["state"]["kind"], "capture");
        assert_eq!(json["state"]["target"], "todo");
        assert_eq!(json["state"]["body"], "Renew passport");
        assert_eq!(json["state"]["priority"], 1);
        assert_eq!(json["state"]["datePhrases"][0]["role"], "due");
        assert_eq!(json["state"]["datePhrases"][0]["source"], "friday");
        assert_eq!(json["state"]["missingRequired"][0], "body");
        assert_eq!(json["state"]["recentTags"][0], "errands");
    }

    #[test]
    fn refine_state_serializes_with_kind() {
        let req = MenuSyntaxAiRequest {
            raw_input: ":type:script database".into(),
            state: MenuSyntaxAiState::Refine {
                free_text: "database".into(),
                predicates: vec!["type:script".into()],
                result_count_hint: Some(0),
            },
        };
        let json = serde_json::to_value(&req).expect("serde");
        assert_eq!(json["state"]["kind"], "refine");
        assert_eq!(json["state"]["freeText"], "database");
        assert_eq!(json["state"]["predicates"][0], "type:script");
        assert_eq!(json["state"]["resultCountHint"], 0);
    }

    #[test]
    fn command_state_serializes_with_kind() {
        let req = MenuSyntaxAiRequest {
            raw_input: ">deploy --".into(),
            state: MenuSyntaxAiState::Command {
                head: "deploy".into(),
                fields: vec![("env".into(), "prod".into())],
                argv: vec![],
                recent_argv: vec!["prod".into(), "--dry-run".into()],
            },
        };
        let json = serde_json::to_value(&req).expect("serde");
        assert_eq!(json["state"]["kind"], "command");
        assert_eq!(json["state"]["head"], "deploy");
        assert_eq!(json["state"]["recentArgv"][1], "--dry-run");
    }

    #[test]
    fn parse_add_tag_response() {
        let json = r#"{
            "kind": "addTag",
            "tag": "errands",
            "title": "Add an errands tag?",
            "acceptLabel": "Add #errands"
        }"#;
        let r = parse_response(json).expect("parse");
        assert_eq!(
            r,
            MenuSyntaxAiResponse::AddTag {
                tag: "errands".into(),
                title: "Add an errands tag?".into(),
                accept_label: "Add #errands".into(),
            }
        );
    }

    #[test]
    fn parse_add_date_response() {
        let json = r#"{
            "kind": "addDate",
            "key": "start",
            "phrase": "friday 2pm",
            "title": "Schedule for Friday 2 PM?",
            "acceptLabel": "Add start:\"friday 2pm\""
        }"#;
        let r = parse_response(json).expect("parse");
        match r {
            MenuSyntaxAiResponse::AddDate {
                key,
                phrase,
                title,
                accept_label,
            } => {
                assert_eq!(key, "start");
                assert_eq!(phrase, "friday 2pm");
                assert_eq!(title, "Schedule for Friday 2 PM?");
                assert_eq!(accept_label, "Add start:\"friday 2pm\"");
            }
            other => panic!("expected AddDate, got {other:?}"),
        }
    }

    #[test]
    fn parse_rewrite_input_response() {
        let json = r#"{
            "kind": "rewriteInput",
            "rewrite": ">deploy -- prod --dry-run",
            "title": "Run deploy with dry-run?",
            "acceptLabel": "Apply"
        }"#;
        let r = parse_response(json).expect("parse");
        match r {
            MenuSyntaxAiResponse::RewriteInput { rewrite, .. } => {
                assert_eq!(rewrite, ">deploy -- prod --dry-run");
            }
            other => panic!("expected RewriteInput, got {other:?}"),
        }
    }

    #[test]
    fn parse_no_suggestion_response() {
        let json = r#"{"kind": "noSuggestion", "reason": "input is already complete"}"#;
        let r = parse_response(json).expect("parse");
        assert_eq!(
            r,
            MenuSyntaxAiResponse::NoSuggestion {
                reason: "input is already complete".into()
            }
        );
    }

    #[test]
    fn parse_empty_returns_empty_input_error() {
        assert_eq!(parse_response(""), Err(AiParseError::EmptyInput));
        assert_eq!(parse_response("   \t\n"), Err(AiParseError::EmptyInput));
    }

    #[test]
    fn parse_garbage_returns_invalid_json_error() {
        let r = parse_response("not json");
        match r {
            Err(AiParseError::InvalidJson(_)) => {}
            other => panic!("expected InvalidJson, got {other:?}"),
        }
    }

    #[test]
    fn missing_required_labels_uses_field_requirement_labels() {
        let labels = missing_required_labels(&[
            FieldRequirement::Body,
            FieldRequirement::AnyDate,
            FieldRequirement::Kv("amount".into()),
        ]);
        assert_eq!(labels, vec!["body", "date", "amount"]);
    }

    #[test]
    fn round_trip_capture_request_through_serde() {
        let req = MenuSyntaxAiRequest {
            raw_input: ";cal Design review".into(),
            state: MenuSyntaxAiState::Capture {
                target: "cal".into(),
                body: "Design review".into(),
                tags: vec![],
                priority: None,
                url: None,
                duration: None,
                kv: vec![],
                date_phrases: vec![],
                missing_required: vec!["date".into()],
                recent_tags: vec![],
            },
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: MenuSyntaxAiRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, req);
    }
}
