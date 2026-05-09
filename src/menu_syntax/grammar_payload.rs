//! Run 14 Pass 5 — unified `GrammarPayload` shape.
//!
//! Story: `grammar-unified-payload-type`. User directive
//! 2026-04-26T00:50Z: every Power Syntax token forms a key:value pair
//! into an object payload destined for the script. Today three
//! independent parsers produce three independent types
//! ([`CaptureInvocation`], [`ArgvInvocation`], [`AdvancedQuery`]). This
//! type is the canonical lossless conversion target so future
//! per-script HISTORY (tag pool, key-value pool) and autocomplete UI
//! consume one shape regardless of which surface produced it.
//!
//! Oracle (gpt-5.4-pro, slug `grammar-unified-payload-design`)
//! recommended the shape below. Critical risk to avoid (per oracle):
//! dual representation drift — never store the same tag/date both as
//! a typed entry AND as a generic `FieldEntry`. Tags live in `tags`,
//! dates live in `dates`, KV fields live in `fields`. A future
//! `record_payload` will iterate the three channels independently.

use serde::{Deserialize, Serialize};

use super::payload::{
    AdvancedQuery, ArgvInvocation, CaptureAlias, CaptureInvocation, DateRole, Predicate,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarVerb {
    Capture,
    Command,
    Refine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarSurface {
    /// Capture prefix payload v1 surface. This serializes as `plus` for
    /// compatibility even when the user typed canonical `;target body`.
    Plus,
    /// keyword-aliased capture (e.g. `todo Buy milk` if `todo` is registered as a keyword)
    Keyword,
    /// `!head -- argv` command
    Bang,
    /// `:type:x #tag` refine
    Colon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
    /// Free-form `key:value` typed by the user (e.g. `priority:1`).
    Free,
    /// Schema-derived first-class field (e.g. capture `url`/`duration`/`priority`).
    Schema,
    /// Refine-query predicate (`:type:script`, `:source:plugin`).
    Query,
    /// Metadata path predicate (`:meta.foo:bar`).
    Meta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagEntry {
    pub value: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldEntry {
    pub key: String,
    pub value: String,
    pub kind: FieldKind,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateEntry {
    pub role: DateRole,
    pub source: String,
    pub source_span: (usize, usize),
}

/// Canonical payload assembled by every Power Syntax parser.
///
/// `version` lets the JSONL history files (a future story) survive
/// schema changes without re-encoding existing rows. Bump `version`
/// whenever a field is renamed or removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrammarPayload {
    pub version: u8,
    pub verb: GrammarVerb,
    pub surface: GrammarSurface,
    /// Capture target (`todo`), command head (`deploy`), or `""` for refine.
    pub target: String,
    /// Original raw input the parser was given.
    pub raw: String,
    /// User-typed body — capture body, refine free-text, or `""` for command.
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dates: Vec<DateEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub argv: Vec<String>,
}

pub const GRAMMAR_PAYLOAD_VERSION: u8 = 1;

impl From<&CaptureInvocation> for GrammarPayload {
    fn from(c: &CaptureInvocation) -> Self {
        let mut fields: Vec<FieldEntry> =
            c.kv.iter()
                .map(|(k, v)| FieldEntry {
                    key: k.clone(),
                    value: v.clone(),
                    kind: FieldKind::Free,
                    negated: false,
                })
                .collect();
        if let Some(url) = &c.url {
            fields.push(FieldEntry {
                key: "url".into(),
                value: url.clone(),
                kind: FieldKind::Schema,
                negated: false,
            });
        }
        if let Some(duration) = &c.duration {
            fields.push(FieldEntry {
                key: "duration".into(),
                value: duration.clone(),
                kind: FieldKind::Schema,
                negated: false,
            });
        }
        if let Some(p) = c.priority {
            fields.push(FieldEntry {
                key: "priority".into(),
                value: p.to_string(),
                kind: FieldKind::Schema,
                negated: false,
            });
        }
        GrammarPayload {
            version: GRAMMAR_PAYLOAD_VERSION,
            verb: GrammarVerb::Capture,
            surface: match c.alias_form {
                CaptureAlias::CapturePrefix => GrammarSurface::Plus,
                CaptureAlias::Keyword => GrammarSurface::Keyword,
            },
            target: c.target.clone(),
            raw: c.raw.clone(),
            text: c.body.clone(),
            tags: c
                .tags
                .iter()
                .map(|t| TagEntry {
                    value: t.clone(),
                    negated: false,
                })
                .collect(),
            fields,
            dates: c
                .date_phrases
                .iter()
                .map(|d| DateEntry {
                    role: d.role.clone(),
                    source: d.source.clone(),
                    source_span: d.source_span,
                })
                .collect(),
            argv: Vec::new(),
        }
    }
}

impl From<&ArgvInvocation> for GrammarPayload {
    fn from(a: &ArgvInvocation) -> Self {
        let fields = a
            .fields
            .iter()
            .map(|(k, v)| FieldEntry {
                key: k.clone(),
                value: v.clone(),
                kind: FieldKind::Free,
                negated: false,
            })
            .collect();
        GrammarPayload {
            version: GRAMMAR_PAYLOAD_VERSION,
            verb: GrammarVerb::Command,
            surface: GrammarSurface::Bang,
            target: a.head.clone(),
            raw: a.raw.clone(),
            text: String::new(),
            tags: a
                .tags
                .iter()
                .map(|t| TagEntry {
                    value: t.clone(),
                    negated: false,
                })
                .collect(),
            fields,
            dates: Vec::new(),
            argv: a.argv.clone(),
        }
    }
}

impl From<&AdvancedQuery> for GrammarPayload {
    fn from(q: &AdvancedQuery) -> Self {
        let mut tags: Vec<TagEntry> = Vec::new();
        let mut fields: Vec<FieldEntry> = Vec::new();
        for predicate in &q.predicates {
            push_predicate(predicate, false, &mut tags, &mut fields);
        }
        GrammarPayload {
            version: GRAMMAR_PAYLOAD_VERSION,
            verb: GrammarVerb::Refine,
            surface: GrammarSurface::Colon,
            target: String::new(),
            raw: q.raw.clone(),
            text: q.free_text.clone(),
            tags,
            fields,
            dates: Vec::new(),
            argv: Vec::new(),
        }
    }
}

fn push_predicate(
    pred: &Predicate,
    negated: bool,
    tags: &mut Vec<TagEntry>,
    fields: &mut Vec<FieldEntry>,
) {
    match pred {
        Predicate::Tag(value) => tags.push(TagEntry {
            value: value.clone(),
            negated,
        }),
        Predicate::Negate(inner) => push_predicate(inner, true, tags, fields),
        Predicate::MetaPath { path, value } => fields.push(FieldEntry {
            key: path.clone(),
            value: value.clone(),
            kind: FieldKind::Meta,
            negated,
        }),
        Predicate::Type(kind) => fields.push(FieldEntry {
            key: "type".into(),
            value: format!("{kind:?}").to_lowercase(),
            kind: FieldKind::Query,
            negated,
        }),
        Predicate::HasShortcut(_) => fields.push(FieldEntry {
            key: "shortcut".into(),
            value: format!("{pred:?}"),
            kind: FieldKind::Query,
            negated,
        }),
        Predicate::Source(value)
        | Predicate::Plugin(value)
        | Predicate::Name(value)
        | Predicate::Desc(value)
        | Predicate::Alias(value)
        | Predicate::Has(value) => {
            let key = match pred {
                Predicate::Source(_) => "source",
                Predicate::Plugin(_) => "plugin",
                Predicate::Name(_) => "name",
                Predicate::Desc(_) => "desc",
                Predicate::Alias(_) => "alias",
                Predicate::Has(_) => "has",
                _ => unreachable!(),
            };
            fields.push(FieldEntry {
                key: key.into(),
                value: value.clone(),
                kind: FieldKind::Query,
                negated,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::payload::{
        AdvancedQuery, ArgvInvocation, ArtifactKind, CaptureAlias, CaptureInvocation, DatePhrase,
        Predicate,
    };
    use super::*;

    fn sample_capture() -> CaptureInvocation {
        CaptureInvocation {
            target: "todo".into(),
            alias_form: CaptureAlias::CapturePrefix,
            body: "Renew passport".into(),
            tags: vec!["errands".into()],
            priority: Some(1),
            url: None,
            duration: None,
            kv: vec![("project".into(), "home".into())],
            date_phrases: vec![DatePhrase {
                role: DateRole::Due,
                source: "tomorrow".into(),
                source_span: (29, 38),
            }],
            raw: ";todo Renew passport #errands p1 due:tomorrow project:home".into(),
        }
    }

    #[test]
    fn capture_invocation_round_trip_through_grammar_payload() {
        let cap = sample_capture();
        let payload: GrammarPayload = (&cap).into();
        assert_eq!(payload.verb, GrammarVerb::Capture);
        assert_eq!(payload.surface, GrammarSurface::Plus);
        assert_eq!(payload.target, "todo");
        assert_eq!(payload.text, "Renew passport");
        assert_eq!(payload.tags.len(), 1);
        assert_eq!(payload.tags[0].value, "errands");
        assert!(!payload.tags[0].negated);
        // priority + project = 2 fields; capture has no url/duration in sample.
        let project = payload.fields.iter().find(|f| f.key == "project").unwrap();
        assert_eq!(project.kind, FieldKind::Free);
        let priority = payload.fields.iter().find(|f| f.key == "priority").unwrap();
        assert_eq!(priority.value, "1");
        assert_eq!(priority.kind, FieldKind::Schema);
        assert_eq!(payload.dates.len(), 1);
        assert_eq!(payload.dates[0].role, DateRole::Due);
        assert!(payload.argv.is_empty());
    }

    #[test]
    fn argv_invocation_maps_command_surface() {
        let argv = ArgvInvocation {
            head: "deploy".into(),
            fields: vec![("env".into(), "prod".into())],
            tags: vec!["release".into()],
            argv: vec!["--dry-run".into()],
            raw: ">deploy env:prod #release -- --dry-run".into(),
        };
        let payload: GrammarPayload = (&argv).into();
        assert_eq!(payload.verb, GrammarVerb::Command);
        assert_eq!(payload.surface, GrammarSurface::Bang);
        assert_eq!(payload.target, "deploy");
        assert_eq!(payload.text, "");
        assert_eq!(payload.fields.len(), 1);
        assert_eq!(payload.fields[0].key, "env");
        assert_eq!(payload.fields[0].kind, FieldKind::Free);
        assert_eq!(payload.tags.len(), 1);
        assert_eq!(payload.argv, vec!["--dry-run"]);
        assert!(payload.dates.is_empty());
    }

    #[test]
    fn advanced_query_carries_negated_tags_and_query_fields() {
        let query = AdvancedQuery {
            free_text: "deploy".into(),
            predicates: vec![
                Predicate::Type(ArtifactKind::Script),
                Predicate::Tag("work".into()),
                Predicate::Negate(Box::new(Predicate::Tag("archived".into()))),
            ],
            raw: ":type:script #work :-tag:archived deploy".into(),
        };
        let payload: GrammarPayload = (&query).into();
        assert_eq!(payload.verb, GrammarVerb::Refine);
        assert_eq!(payload.surface, GrammarSurface::Colon);
        assert_eq!(payload.text, "deploy");
        assert_eq!(payload.tags.len(), 2);
        let archived = payload.tags.iter().find(|t| t.value == "archived").unwrap();
        assert!(archived.negated);
        let work = payload.tags.iter().find(|t| t.value == "work").unwrap();
        assert!(!work.negated);
        let type_field = payload.fields.iter().find(|f| f.key == "type").unwrap();
        assert_eq!(type_field.kind, FieldKind::Query);
        assert_eq!(type_field.value, "script");
    }

    #[test]
    fn serializes_to_camelcase_json_with_omitted_empty_collections() {
        let payload = GrammarPayload {
            version: GRAMMAR_PAYLOAD_VERSION,
            verb: GrammarVerb::Command,
            surface: GrammarSurface::Bang,
            target: "deploy".into(),
            raw: ">deploy".into(),
            text: String::new(),
            tags: Vec::new(),
            fields: Vec::new(),
            dates: Vec::new(),
            argv: Vec::new(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"verb\":\"command\""));
        assert!(json.contains("\"surface\":\"bang\""));
        assert!(json.contains("\"target\":\"deploy\""));
        assert!(!json.contains("\"tags\""));
        assert!(!json.contains("\"fields\""));
        assert!(!json.contains("\"dates\""));
        assert!(!json.contains("\"argv\""));
    }
}
