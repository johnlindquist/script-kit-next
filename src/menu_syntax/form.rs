use serde::{Deserialize, Serialize};

use super::capture_schema::{CaptureFieldSchema, FieldRequirement, ValidationResult};
use super::history::{TagFrequency, ValueFrequency};
use super::payload::{CaptureAlias, CaptureInvocation, DateRole};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuSyntaxFormFieldKind {
    Body,
    Tags,
    Priority,
    Date,
    Url,
    Duration,
    Object,
    KeyValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxFormSuggestion {
    pub value: String,
    pub label: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxFormFieldSnapshot {
    pub id: String,
    pub label: String,
    pub kind: MenuSyntaxFormFieldKind,
    pub value: String,
    pub placeholder: String,
    pub required: bool,
    pub satisfied: bool,
    pub focused: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<MenuSyntaxFormSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxFormSnapshot {
    pub target: String,
    pub focused_index: usize,
    pub tab_ai_disabled: bool,
    pub sync_source: String,
    pub can_submit: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<MenuSyntaxFormFieldSnapshot>,
}

#[derive(Debug, Clone, Default)]
pub struct MenuSyntaxFormSuggestionPools {
    pub tags: Vec<TagFrequency>,
    pub priority_values: Vec<String>,
    pub date_values: Vec<ValueFrequency>,
    pub url_values: Vec<ValueFrequency>,
}

pub fn empty_capture_invocation(target: &str, raw: &str) -> CaptureInvocation {
    let trimmed = raw.trim_start();
    CaptureInvocation {
        target: target.to_ascii_lowercase(),
        alias_form: if trimmed.starts_with(';') || trimmed.starts_with('+') {
            CaptureAlias::CapturePrefix
        } else {
            CaptureAlias::Keyword
        },
        body: String::new(),
        tags: Vec::new(),
        priority: None,
        url: None,
        duration: None,
        kv: Vec::new(),
        date_phrases: Vec::new(),
        raw: raw.to_string(),
    }
}

pub fn build_capture_form_snapshot(
    schema: &CaptureFieldSchema,
    invocation: &CaptureInvocation,
    focused_index: usize,
    validation: &ValidationResult,
    pools: MenuSyntaxFormSuggestionPools,
) -> MenuSyntaxFormSnapshot {
    let mut requirements = schema.required.clone();
    for optional in &schema.optional {
        if !requirements.contains(optional) {
            requirements.push(optional.clone());
        }
    }

    let mut fields = requirements
        .into_iter()
        .filter_map(|requirement| {
            form_field_for_requirement(schema, invocation, requirement, &pools)
        })
        .collect::<Vec<_>>();

    let focused_index = if fields.is_empty() {
        0
    } else {
        focused_index.min(fields.len() - 1)
    };
    for (index, field) in fields.iter_mut().enumerate() {
        field.focused = index == focused_index;
    }

    MenuSyntaxFormSnapshot {
        target: schema.target.clone(),
        focused_index,
        tab_ai_disabled: true,
        sync_source: "mainInput".to_string(),
        can_submit: matches!(validation, ValidationResult::Ready),
        fields,
    }
}

pub fn apply_capture_form_field_edit(
    invocation: &CaptureInvocation,
    field_id: &str,
    value: &str,
) -> Option<String> {
    let mut next = invocation.clone();
    let normalized = value.trim();

    match field_id {
        "body" => {
            next.body = normalized.to_string();
        }
        "tags" => {
            next.tags = parse_tag_field(normalized);
        }
        "priority" => {
            next.priority = parse_priority_field(normalized);
        }
        "date" => {
            next.date_phrases = parse_date_field(
                normalized,
                next.date_phrases.first().map(|d| d.role.clone()),
            );
        }
        "url" => {
            next.url = (!normalized.is_empty()).then(|| normalized.to_string());
        }
        "duration" => {
            next.duration = (!normalized.is_empty()).then(|| normalized.to_string());
        }
        "object" => {
            if !normalized.is_empty() {
                next.body = append_or_replace_object_token(&next.body, normalized);
            }
        }
        "trigger" => {
            upsert_kv(&mut next.kv, "trigger", normalized);
        }
        id if id.starts_with("kv:") => {
            let key = id.trim_start_matches("kv:");
            if key.is_empty() {
                return None;
            }
            upsert_kv(&mut next.kv, key, normalized);
        }
        _ => return None,
    }

    Some(serialize_capture_invocation(&next))
}

fn serialize_capture_invocation(invocation: &CaptureInvocation) -> String {
    let mut parts = Vec::new();
    let head = if invocation.raw.starts_with('+') {
        format!("+{}", invocation.target)
    } else if matches!(invocation.alias_form, super::payload::CaptureAlias::Keyword) {
        format!("{}:", invocation.target)
    } else {
        format!(";{}", invocation.target)
    };
    parts.push(head);

    if !invocation.body.trim().is_empty() {
        parts.push(invocation.body.trim().to_string());
    }
    for tag in &invocation.tags {
        let tag = tag.trim().trim_start_matches('#');
        if !tag.is_empty() {
            parts.push(format!("#{tag}"));
        }
    }
    if let Some(priority) = invocation.priority {
        parts.push(format!("p{priority}"));
    }
    for phrase in &invocation.date_phrases {
        let source = phrase.source.trim();
        if !source.is_empty() {
            parts.push(format!(
                "{}:{}",
                date_role_token(phrase.role.clone()),
                quote_token_value(source)
            ));
        }
    }
    if let Some(url) = invocation
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    {
        parts.push(url.to_string());
    }
    if let Some(duration) = invocation
        .duration
        .as_deref()
        .map(str::trim)
        .filter(|duration| !duration.is_empty())
    {
        parts.push(format!("for:{}", quote_token_value(duration)));
    }
    for (key, value) in &invocation.kv {
        let key = key.trim();
        let value = value.trim();
        if !key.is_empty() && !value.is_empty() {
            parts.push(format!("{key}:{}", quote_token_value(value)));
        }
    }

    parts.join(" ")
}

fn parse_tag_field(value: &str) -> Vec<String> {
    value
        .split(|ch: char| ch.is_ascii_whitespace() || ch == ',')
        .filter_map(|raw| {
            let tag = raw.trim().trim_start_matches('#');
            (!tag.is_empty()).then(|| tag.to_string())
        })
        .collect()
}

fn parse_priority_field(value: &str) -> Option<u8> {
    let trimmed = value.trim().trim_start_matches('p').trim_start_matches('P');
    let priority = trimmed.parse::<u8>().ok()?;
    (1..=4).contains(&priority).then_some(priority)
}

fn parse_date_field(
    value: &str,
    fallback_role: Option<DateRole>,
) -> Vec<super::payload::DatePhrase> {
    if value.trim().is_empty() {
        return Vec::new();
    }
    value
        .split(',')
        .filter_map(|source| {
            let source = source.trim();
            (!source.is_empty()).then(|| super::payload::DatePhrase {
                role: fallback_role.clone().unwrap_or(DateRole::Due),
                source: source.to_string(),
                source_span: (0, 0),
            })
        })
        .collect()
}

fn upsert_kv(kv: &mut Vec<(String, String)>, key: &str, value: &str) {
    if let Some((_, existing)) = kv
        .iter_mut()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
    {
        *existing = value.to_string();
        return;
    }
    if !value.is_empty() {
        kv.push((key.to_string(), value.to_string()));
    }
}

fn append_or_replace_object_token(body: &str, value: &str) -> String {
    let token = value.trim();
    if token.is_empty() {
        return body.trim().to_string();
    }
    let token = if token.starts_with('@') {
        token.to_string()
    } else {
        format!("@{token}")
    };
    let mut parts = body
        .split_whitespace()
        .filter(|part| !part.starts_with('@'))
        .map(str::to_string)
        .collect::<Vec<_>>();
    parts.push(token);
    parts.join(" ")
}

fn date_role_token(role: DateRole) -> &'static str {
    match role {
        DateRole::At => "at",
        DateRole::Start => "start",
        DateRole::End => "end",
        DateRole::Due | DateRole::Inferred => "due",
    }
}

fn quote_token_value(value: &str) -> String {
    if value
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || ch == '"' || ch == '\'')
    {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

fn form_field_for_requirement(
    schema: &CaptureFieldSchema,
    invocation: &CaptureInvocation,
    requirement: FieldRequirement,
    pools: &MenuSyntaxFormSuggestionPools,
) -> Option<MenuSyntaxFormFieldSnapshot> {
    let required = schema.required.contains(&requirement);
    let satisfied = requirement.is_satisfied(invocation);
    let (id, label, kind, value, placeholder, suggestions) = match &requirement {
        FieldRequirement::Body => (
            "body".to_string(),
            "Task".to_string(),
            MenuSyntaxFormFieldKind::Body,
            invocation.body.clone(),
            format!("What should ;{} capture?", schema.target),
            Vec::new(),
        ),
        FieldRequirement::Tag => (
            "tags".to_string(),
            "Tags".to_string(),
            MenuSyntaxFormFieldKind::Tags,
            invocation
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" "),
            "#project #errands".to_string(),
            pools
                .tags
                .iter()
                .take(6)
                .map(|tag| MenuSyntaxFormSuggestion {
                    value: format!("#{}", tag.tag),
                    label: format!("#{} ({})", tag.tag, tag.count),
                    source: "tagHistory".to_string(),
                })
                .collect(),
        ),
        FieldRequirement::Priority => (
            "priority".to_string(),
            "Priority".to_string(),
            MenuSyntaxFormFieldKind::Priority,
            invocation
                .priority
                .map(|priority| format!("p{priority}"))
                .unwrap_or_default(),
            "p1, p2, p3, or p4".to_string(),
            pools
                .priority_values
                .iter()
                .map(|value| MenuSyntaxFormSuggestion {
                    value: value.clone(),
                    label: value.clone(),
                    source: "schema".to_string(),
                })
                .collect(),
        ),
        FieldRequirement::AnyDate | FieldRequirement::DateRole(_) => (
            "date".to_string(),
            date_label(&requirement),
            MenuSyntaxFormFieldKind::Date,
            invocation
                .date_phrases
                .iter()
                .map(|date| date.source.clone())
                .collect::<Vec<_>>()
                .join(", "),
            "tomorrow 3pm, next Friday, in 2 hours".to_string(),
            pools
                .date_values
                .iter()
                .take(5)
                .map(|value| MenuSyntaxFormSuggestion {
                    value: value.value.clone(),
                    label: value.value.clone(),
                    source: "dateHistory".to_string(),
                })
                .collect(),
        ),
        FieldRequirement::Url => (
            "url".to_string(),
            "URL".to_string(),
            MenuSyntaxFormFieldKind::Url,
            invocation.url.clone().unwrap_or_default(),
            "https://example.com".to_string(),
            pools
                .url_values
                .iter()
                .take(5)
                .map(|value| MenuSyntaxFormSuggestion {
                    value: value.value.clone(),
                    label: value.value.clone(),
                    source: "urlHistory".to_string(),
                })
                .collect(),
        ),
        FieldRequirement::Duration => (
            "duration".to_string(),
            "Duration".to_string(),
            MenuSyntaxFormFieldKind::Duration,
            invocation.duration.clone().unwrap_or_default(),
            "30m, 2h, 1d".to_string(),
            Vec::new(),
        ),
        FieldRequirement::ObjectSelection => (
            "object".to_string(),
            "Existing item".to_string(),
            MenuSyntaxFormFieldKind::Object,
            String::new(),
            "@ to search existing todos, notes, links, or snippets".to_string(),
            Vec::new(),
        ),
        FieldRequirement::Kv(key) => (
            format!("kv:{key}"),
            key.clone(),
            MenuSyntaxFormFieldKind::KeyValue,
            invocation
                .kv
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
                .map(|(_, value)| value.clone())
                .unwrap_or_default(),
            format!("{key}:value"),
            Vec::new(),
        ),
        FieldRequirement::SnippetTriggerOrSelection => (
            "trigger".to_string(),
            "Trigger".to_string(),
            MenuSyntaxFormFieldKind::KeyValue,
            invocation
                .kv
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case("trigger"))
                .map(|(_, value)| value.clone())
                .unwrap_or_default(),
            "trigger:shortcut or @existing".to_string(),
            Vec::new(),
        ),
    };

    Some(MenuSyntaxFormFieldSnapshot {
        id,
        label,
        kind,
        value,
        placeholder,
        required,
        satisfied,
        focused: false,
        suggestions,
    })
}

fn date_label(requirement: &FieldRequirement) -> String {
    match requirement {
        FieldRequirement::DateRole(DateRole::Due) => "Due".to_string(),
        FieldRequirement::DateRole(DateRole::At) => "At".to_string(),
        FieldRequirement::DateRole(DateRole::Start) => "Start".to_string(),
        FieldRequirement::DateRole(DateRole::End) => "End".to_string(),
        _ => "Date".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::parse_capture;
    use crate::menu_syntax::capture_schema::{builtin_schema, validate};

    fn todo_invocation(raw: &str) -> CaptureInvocation {
        match parse_capture(raw) {
            crate::menu_syntax::capture::CaptureParse::Ok(invocation) => invocation,
            crate::menu_syntax::capture::CaptureParse::Incomplete(_) => {
                panic!("expected capture invocation")
            }
        }
    }

    #[test]
    fn todo_form_projects_schema_fields_and_values() {
        let schema = builtin_schema("todo").expect("todo schema");
        let invocation = todo_invocation(";todo Renew passport #errands p1 due:tomorrow");
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            1,
            &validation,
            MenuSyntaxFormSuggestionPools {
                priority_values: vec!["p1".to_string(), "p2".to_string()],
                ..Default::default()
            },
        );

        assert_eq!(snapshot.target, "todo");
        assert!(snapshot.tab_ai_disabled);
        assert_eq!(snapshot.focused_index, 1);
        assert!(snapshot
            .fields
            .iter()
            .any(|field| field.id == "body" && field.value == "Renew passport" && field.required));
        assert!(snapshot
            .fields
            .iter()
            .any(|field| field.id == "tags" && field.value == "#errands" && field.focused));
        assert!(snapshot.fields.iter().any(|field| field.id == "priority"
            && field.value == "p1"
            && !field.suggestions.is_empty()));
    }

    #[test]
    fn editing_tags_rewrites_canonical_capture_text() {
        let invocation = todo_invocation(";todo Renew passport #errands p1 due:tomorrow");
        let rewritten = apply_capture_form_field_edit(&invocation, "tags", "#travel work")
            .expect("tags edit should serialize");

        assert_eq!(
            rewritten,
            ";todo Renew passport #travel #work p1 due:tomorrow"
        );
    }

    #[test]
    fn editing_date_preserves_source_phrase_not_iso() {
        let invocation = todo_invocation(";todo Renew passport #errands p1 due:tomorrow");
        let rewritten = apply_capture_form_field_edit(&invocation, "date", "Friday 2pm")
            .expect("date edit should serialize");

        assert_eq!(
            rewritten,
            ";todo Renew passport #errands p1 due:\"Friday 2pm\""
        );
    }

    #[test]
    fn editing_kv_field_uses_power_syntax_token() {
        let invocation = todo_invocation(";todo Renew passport #errands p1");
        let rewritten = apply_capture_form_field_edit(&invocation, "kv:project", "travel docs")
            .expect("kv edit should serialize");

        assert_eq!(
            rewritten,
            ";todo Renew passport #errands p1 project:\"travel docs\""
        );
    }

    #[test]
    fn empty_invocations_build_forms_for_bare_handlers() {
        for target in ["todo", "note", "link", "snippet", "cal", "social"] {
            let schema = builtin_schema(target).unwrap_or_else(|| panic!("{target} schema"));
            let invocation = empty_capture_invocation(target, &format!(";{target}"));
            let validation = validate(&invocation, &schema);
            let snapshot = build_capture_form_snapshot(
                &schema,
                &invocation,
                0,
                &validation,
                MenuSyntaxFormSuggestionPools::default(),
            );

            assert_eq!(snapshot.target, target);
            assert!(snapshot.tab_ai_disabled);
            assert!(
                !snapshot.can_submit,
                "bare {target} should expose fields but remain incomplete"
            );
            assert!(
                !snapshot.fields.is_empty(),
                "bare {target} should render form fields"
            );
            assert!(
                snapshot.fields.iter().any(|field| field.required),
                "bare {target} should surface required fields"
            );
        }
    }
}
