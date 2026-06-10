use serde::{Deserialize, Serialize};

use super::capture_schema::{CaptureFieldSchema, FieldRequirement, ValidationResult};
use super::history::{TagFrequency, ValueFrequency};
use super::payload::{CaptureAlias, CaptureInvocation, DateRole};
use super::snippet_scriptlet::{parse_snippet_scriptlet_capture, SnippetScriptletOperation};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxFormFieldSnapshot {
    pub id: String,
    pub label: String,
    pub kind: MenuSyntaxFormFieldKind,
    pub value: String,
    pub placeholder: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub multiline: bool,
    pub required: bool,
    pub satisfied: bool,
    pub focused: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub suggestion_query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_suggestion_index: Option<usize>,
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
    pub objects: Vec<crate::menu_syntax::ObjectSelectorCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSyntaxFormSuggestionApplication {
    pub next_field_value: String,
}

pub fn empty_capture_invocation(target: &str, raw: &str) -> CaptureInvocation {
    let trimmed = raw.trim_start();
    CaptureInvocation {
        target: target.to_ascii_lowercase(),
        alias_form: if trimmed.starts_with(';')
            || trimmed.starts_with('+')
            || raw_is_postfix_capture(raw, target)
        {
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
            if !normalized.is_empty() && parse_priority_field(normalized).is_none() {
                return None;
            }
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
            next.body = if normalized.is_empty() {
                remove_object_tokens(&next.body)
            } else {
                append_or_replace_object_token(&next.body, normalized)
            };
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

pub fn apply_menu_syntax_form_suggestion(
    field: &MenuSyntaxFormFieldSnapshot,
    suggestion: &MenuSyntaxFormSuggestion,
) -> Option<MenuSyntaxFormSuggestionApplication> {
    let next_field_value = match field.kind {
        MenuSyntaxFormFieldKind::Tags => {
            let range = active_completion_range(&field.value, true)?;
            let mut next = String::new();
            next.push_str(&field.value[..range.start]);
            next.push_str(&normalize_tag_suggestion_value(&suggestion.value));
            if !next.ends_with(' ') {
                next.push(' ');
            }
            next.push_str(&field.value[range.end..]);
            next.trim_end().to_string()
        }
        MenuSyntaxFormFieldKind::Object => suggestion.value.clone(),
        MenuSyntaxFormFieldKind::Body
        | MenuSyntaxFormFieldKind::Priority
        | MenuSyntaxFormFieldKind::Date
        | MenuSyntaxFormFieldKind::Url
        | MenuSyntaxFormFieldKind::Duration
        | MenuSyntaxFormFieldKind::KeyValue => suggestion.value.clone(),
    };

    Some(MenuSyntaxFormSuggestionApplication { next_field_value })
}

fn active_completion_range(value: &str, allow_comma: bool) -> Option<std::ops::Range<usize>> {
    let end = value.len();
    let start = value
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| {
            (ch.is_ascii_whitespace() || (allow_comma && ch == ',')).then_some(idx + ch.len_utf8())
        })
        .unwrap_or(0);
    Some(start..end)
}

fn active_completion_query(value: &str, allow_comma: bool) -> String {
    active_completion_range(value, allow_comma)
        .map(|range| value[range].trim().to_string())
        .unwrap_or_default()
}

fn normalize_tag_suggestion_value(value: &str) -> String {
    let tag = value.trim().trim_start_matches('#');
    if tag.is_empty() {
        String::new()
    } else {
        format!("#{tag}")
    }
}

fn serialize_capture_invocation(invocation: &CaptureInvocation) -> String {
    if should_serialize_snippet_body_after_delimiter(invocation) {
        return serialize_snippet_create_capture_invocation(invocation);
    }

    let mut parts = Vec::new();
    parts.push(capture_invocation_head(invocation));

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
        let has_body = !invocation.body.trim().is_empty();
        if !invocation.target.eq_ignore_ascii_case("link")
            || !has_body
            || invocation.body.trim() != url
        {
            parts.push(url.to_string());
        }
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

fn capture_invocation_head(invocation: &CaptureInvocation) -> String {
    if raw_is_postfix_capture(&invocation.raw, &invocation.target) {
        // A4 decision (2026-06-09): postfix `todo;` is the canonical capture
        // spelling; rewrites must not flip it back to `;todo`/`todo:`.
        format!("{};", invocation.target)
    } else if invocation.raw.starts_with('+') {
        format!("+{}", invocation.target)
    } else if matches!(invocation.alias_form, super::payload::CaptureAlias::Keyword) {
        format!("{}:", invocation.target)
    } else {
        format!(";{}", invocation.target)
    }
}

fn raw_is_postfix_capture(raw: &str, target: &str) -> bool {
    let trimmed = raw.trim_start();
    trimmed.len() > target.len()
        && trimmed.is_char_boundary(target.len())
        && trimmed[..target.len()].eq_ignore_ascii_case(target)
        && trimmed[target.len()..].starts_with(';')
}

fn should_serialize_snippet_body_after_delimiter(invocation: &CaptureInvocation) -> bool {
    if !invocation.target.eq_ignore_ascii_case("snippet") {
        return false;
    }
    parse_snippet_scriptlet_capture(invocation)
        .map(|draft| matches!(draft.operation, SnippetScriptletOperation::Create))
        .unwrap_or(false)
}

fn serialize_snippet_create_capture_invocation(invocation: &CaptureInvocation) -> String {
    let mut parts = Vec::new();
    parts.push(capture_invocation_head(invocation));

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

    let body = invocation.body.trim();
    if !body.is_empty() {
        parts.push("--".to_string());
        parts.push(body.to_string());
    }

    parts.join(" ")
}

fn parse_tag_field(value: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for raw in value.split(|ch: char| ch.is_ascii_whitespace() || ch == ',') {
        let tag = raw.trim().trim_start_matches('#');
        if tag.is_empty() {
            continue;
        }
        if !tags
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(tag))
        {
            tags.push(tag.to_string());
        }
    }
    tags
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

fn first_object_token_from_body(body: &str) -> String {
    body.split_whitespace()
        .find(|part| part.starts_with('@'))
        .unwrap_or_default()
        .to_string()
}

fn first_object_token_from_invocation(invocation: &CaptureInvocation) -> String {
    crate::menu_syntax::object_refs_for_raw_capture(&invocation.target, &invocation.raw)
        .into_iter()
        .find_map(|object_ref| object_ref.token)
        .unwrap_or_else(|| first_object_token_from_body(&invocation.body))
}

fn remove_object_tokens(body: &str) -> String {
    body.split_whitespace()
        .filter(|part| !part.starts_with('@'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn text_matches_query(value: &str, query: &str) -> bool {
    let query = query
        .trim()
        .trim_start_matches(['#', '@'])
        .to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }
    value.to_ascii_lowercase().contains(&query)
}

fn filter_tag_suggestions(
    current_value: &str,
    tags: &[TagFrequency],
) -> (String, Vec<MenuSyntaxFormSuggestion>) {
    let raw_query = active_completion_query(current_value, true);
    let query = raw_query.trim_start_matches('#').to_ascii_lowercase();
    let active_range = active_completion_range(current_value, true);
    let stable_value = active_range
        .map(|range| {
            let mut value = current_value.to_string();
            value.replace_range(range, "");
            value
        })
        .unwrap_or_else(|| current_value.to_string());
    let existing = parse_tag_field(&stable_value);
    let suggestions = tags
        .iter()
        .filter(|tag| {
            !existing
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&tag.tag))
        })
        .filter(|tag| query.is_empty() || tag.tag.to_ascii_lowercase().contains(&query))
        .take(8)
        .map(|tag| MenuSyntaxFormSuggestion {
            value: format!("#{}", tag.tag),
            label: format!("#{} ({})", tag.tag, tag.count),
            source: "tagHistory".to_string(),
            detail: Some("Recent tag".to_string()),
        })
        .collect();
    (raw_query, suggestions)
}

fn value_suggestions(
    current_value: &str,
    source: &str,
    values: &[ValueFrequency],
    limit: usize,
) -> (String, Vec<MenuSyntaxFormSuggestion>) {
    let raw_query = active_completion_query(current_value, false);
    let suggestions = values
        .iter()
        .filter(|value| text_matches_query(&value.value, &raw_query))
        .take(limit)
        .map(|value| MenuSyntaxFormSuggestion {
            value: value.value.clone(),
            label: value.value.clone(),
            source: source.to_string(),
            detail: if value.count > 1 {
                Some(format!("Used {} times", value.count))
            } else {
                None
            },
        })
        .collect();
    (raw_query, suggestions)
}

fn priority_suggestions(
    current_value: &str,
    values: &[String],
) -> (String, Vec<MenuSyntaxFormSuggestion>) {
    let raw_query = active_completion_query(current_value, false);
    let suggestions = values
        .iter()
        .map(|value| MenuSyntaxFormSuggestion {
            value: value.clone(),
            label: value.clone(),
            source: "schema".to_string(),
            detail: None,
        })
        .collect();
    (raw_query, suggestions)
}

fn filter_object_suggestions(
    current_value: &str,
    objects: &[crate::menu_syntax::ObjectSelectorCandidate],
) -> (String, Vec<MenuSyntaxFormSuggestion>) {
    let raw_query = current_value.trim().to_string();
    let query = raw_query
        .trim_start_matches('@')
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| raw_query.trim_start_matches('@'));
    let suggestions = objects
        .iter()
        .filter(|candidate| {
            query.is_empty()
                || crate::menu_syntax::object_selector_candidate_matches(candidate, query)
        })
        .take(8)
        .map(|candidate| MenuSyntaxFormSuggestion {
            value: candidate.token(),
            label: format!("@{}", candidate.label),
            source: format!("object:{}", candidate.kind.as_str()),
            detail: (!candidate.subtitle.is_empty()).then(|| candidate.subtitle.clone()),
        })
        .collect();
    (raw_query, suggestions)
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

fn required_form_label(label: String, required: bool) -> String {
    if required && !label.ends_with(" *") {
        format!("{label} *")
    } else {
        label
    }
}

fn menu_syntax_form_field_is_multiline(
    schema: &CaptureFieldSchema,
    requirement: &FieldRequirement,
) -> bool {
    matches!(requirement, FieldRequirement::Body) && schema.target.eq_ignore_ascii_case("snippet")
}

fn kv_placeholder_for_key(key: &str, schema_target: &str) -> String {
    match key.to_ascii_lowercase().as_str() {
        "name" if schema_target.eq_ignore_ascii_case("snippet") => "Snippet name".to_string(),
        "keyword" if schema_target.eq_ignore_ascii_case("snippet") => {
            "Expansion keyword".to_string()
        }
        "title" if schema_target.eq_ignore_ascii_case("link") => "Link title".to_string(),
        "description" => "Optional description".to_string(),
        "hidden" | "background" | "system" | "fallback" => "true or false".to_string(),
        "tags" | "watch" => "Values separated by spaces or commas".to_string(),
        _ => "Value".to_string(),
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
    let multiline = menu_syntax_form_field_is_multiline(schema, &requirement);
    let (id, label, kind, value, placeholder, suggestion_query, suggestions) = match &requirement {
        FieldRequirement::Body => (
            "body".to_string(),
            if schema.target.eq_ignore_ascii_case("snippet") {
                "Snippet".to_string()
            } else if schema.target.eq_ignore_ascii_case("link") {
                "Title".to_string()
            } else {
                "Task".to_string()
            },
            MenuSyntaxFormFieldKind::Body,
            invocation.body.clone(),
            if schema.target.eq_ignore_ascii_case("snippet") {
                "Text to paste/expand".to_string()
            } else if schema.target.eq_ignore_ascii_case("link") {
                "Optional link title".to_string()
            } else {
                format!("What should ;{} capture?", schema.target)
            },
            String::new(),
            Vec::new(),
        ),
        FieldRequirement::Tag => {
            let value = invocation
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" ");
            let (suggestion_query, suggestions) = filter_tag_suggestions(&value, &pools.tags);
            (
                "tags".to_string(),
                "Tags".to_string(),
                MenuSyntaxFormFieldKind::Tags,
                value,
                "#project #errands".to_string(),
                suggestion_query,
                suggestions,
            )
        }
        FieldRequirement::Priority => {
            let value = invocation
                .priority
                .map(|priority| format!("p{priority}"))
                .unwrap_or_default();
            let (suggestion_query, suggestions) =
                priority_suggestions(&value, &pools.priority_values);
            (
                "priority".to_string(),
                "Priority".to_string(),
                MenuSyntaxFormFieldKind::Priority,
                value,
                "p1, p2, p3, or p4".to_string(),
                suggestion_query,
                suggestions,
            )
        }
        FieldRequirement::AnyDate | FieldRequirement::DateRole(_) => {
            let value = invocation
                .date_phrases
                .iter()
                .map(|date| date.source.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let (suggestion_query, suggestions) =
                value_suggestions(&value, "dateHistory", &pools.date_values, 5);
            (
                "date".to_string(),
                date_label(&requirement),
                MenuSyntaxFormFieldKind::Date,
                value,
                "tomorrow 3pm, next Friday, in 2 hours".to_string(),
                suggestion_query,
                suggestions,
            )
        }
        FieldRequirement::Url => {
            let value = invocation.url.clone().unwrap_or_default();
            let (suggestion_query, suggestions) =
                value_suggestions(&value, "urlHistory", &pools.url_values, 5);
            (
                "url".to_string(),
                "URL".to_string(),
                MenuSyntaxFormFieldKind::Url,
                value,
                "https://example.com".to_string(),
                suggestion_query,
                suggestions,
            )
        }
        FieldRequirement::Duration => (
            "duration".to_string(),
            "Duration".to_string(),
            MenuSyntaxFormFieldKind::Duration,
            invocation.duration.clone().unwrap_or_default(),
            "30m, 2h, 1d".to_string(),
            active_completion_query(invocation.duration.as_deref().unwrap_or_default(), false),
            Vec::new(),
        ),
        FieldRequirement::ObjectSelection => {
            let value = first_object_token_from_invocation(invocation);
            let (suggestion_query, suggestions) = filter_object_suggestions(&value, &pools.objects);
            (
                "object".to_string(),
                "Existing item".to_string(),
                MenuSyntaxFormFieldKind::Object,
                value,
                "@ to search existing todos, notes, links, or snippets".to_string(),
                suggestion_query,
                suggestions,
            )
        }
        FieldRequirement::Kv(key) => {
            if schema.target.eq_ignore_ascii_case("link")
                && (key.eq_ignore_ascii_case("title") || key.eq_ignore_ascii_case("url"))
            {
                return None;
            }
            if (key.eq_ignore_ascii_case("tags") || key.eq_ignore_ascii_case("tag"))
                && (schema.required.contains(&FieldRequirement::Tag)
                    || schema.optional.contains(&FieldRequirement::Tag))
            {
                return None;
            }

            let value = invocation
                .kv
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
                .map(|(_, value)| value.clone())
                .unwrap_or_default();
            (
                format!("kv:{key}"),
                key.clone(),
                MenuSyntaxFormFieldKind::KeyValue,
                value.clone(),
                kv_placeholder_for_key(key, &schema.target),
                active_completion_query(&value, false),
                Vec::new(),
            )
        }
        FieldRequirement::SnippetTriggerOrSelection => {
            let value = invocation
                .kv
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case("trigger"))
                .map(|(_, value)| value.clone())
                .unwrap_or_default();
            (
                "trigger".to_string(),
                "Trigger".to_string(),
                MenuSyntaxFormFieldKind::KeyValue,
                value.clone(),
                "Shortcut or @existing".to_string(),
                active_completion_query(&value, false),
                Vec::new(),
            )
        }
        FieldRequirement::SnippetNameOrSelection => {
            let value = invocation
                .kv
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case("name"))
                .map(|(_, value)| value.clone())
                .unwrap_or_default();
            (
                "kv:name".to_string(),
                "name".to_string(),
                MenuSyntaxFormFieldKind::KeyValue,
                value.clone(),
                "Snippet name".to_string(),
                active_completion_query(&value, false),
                Vec::new(),
            )
        }
    };
    let label = required_form_label(label, required);

    Some(MenuSyntaxFormFieldSnapshot {
        id,
        label,
        kind,
        value,
        placeholder,
        multiline,
        required,
        satisfied,
        focused: false,
        suggestion_query,
        selected_suggestion_index: None,
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
    fn editing_postfix_capture_preserves_postfix_head() {
        // A4 decision (2026-06-09): form field edits on a `todo;` capture
        // must rewrite using the postfix spelling, not `;todo`/`todo:`.
        let parsed = crate::menu_syntax::parse::parse("todo; Renew passport #errands p1");
        let invocation = match parsed {
            crate::menu_syntax::parse::MenuSyntaxParse::Capture(inv) => inv,
            other => panic!("expected postfix capture, got {other:?}"),
        };
        let rewritten = apply_capture_form_field_edit(&invocation, "tags", "#travel")
            .expect("tags edit should serialize");
        assert_eq!(rewritten, "todo; Renew passport #travel p1");
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
    fn tag_suggestions_filter_by_active_hash_fragment() {
        let schema = builtin_schema("todo").expect("todo schema");
        let invocation = todo_invocation(";todo Renew passport #e");
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            1,
            &validation,
            MenuSyntaxFormSuggestionPools {
                tags: vec![
                    TagFrequency {
                        tag: "errands".to_string(),
                        count: 3,
                        last_seen_ts: 3,
                    },
                    TagFrequency {
                        tag: "work".to_string(),
                        count: 2,
                        last_seen_ts: 2,
                    },
                ],
                ..Default::default()
            },
        );
        let tags = snapshot
            .fields
            .iter()
            .find(|field| field.id == "tags")
            .expect("tags field");

        assert_eq!(tags.suggestion_query, "#e");
        assert_eq!(tags.suggestions.len(), 1);
        assert_eq!(tags.suggestions[0].value, "#errands");
    }

    #[test]
    fn accepting_tag_suggestion_rewrites_canonical_capture_text() {
        let invocation = todo_invocation(";todo Renew passport #e");
        let field = MenuSyntaxFormFieldSnapshot {
            id: "tags".to_string(),
            label: "Tags".to_string(),
            kind: MenuSyntaxFormFieldKind::Tags,
            value: "#e".to_string(),
            placeholder: String::new(),
            multiline: false,
            required: false,
            satisfied: true,
            focused: true,
            suggestion_query: "#e".to_string(),
            selected_suggestion_index: Some(0),
            suggestions: vec![],
        };
        let suggestion = MenuSyntaxFormSuggestion {
            value: "#errands".to_string(),
            label: "#errands (3)".to_string(),
            source: "tagHistory".to_string(),
            detail: Some("Recent tag".to_string()),
        };
        let application =
            apply_menu_syntax_form_suggestion(&field, &suggestion).expect("apply suggestion");
        let rewritten =
            apply_capture_form_field_edit(&invocation, "tags", &application.next_field_value)
                .expect("tags edit should serialize");

        assert_eq!(application.next_field_value, "#errands");
        assert_eq!(rewritten, ";todo Renew passport #errands");
    }

    #[test]
    fn editing_tags_dedupes_values() {
        let invocation = todo_invocation(";todo Eat lunch #food");
        let rewritten = apply_capture_form_field_edit(&invocation, "tags", "#food #food")
            .expect("tags edit should serialize");

        assert_eq!(rewritten, ";todo Eat lunch #food");
    }

    #[test]
    fn accepting_same_tag_suggestion_is_idempotent() {
        let invocation = todo_invocation(";todo Eat lunch #food");
        let field = MenuSyntaxFormFieldSnapshot {
            id: "tags".to_string(),
            label: "Tags".to_string(),
            kind: MenuSyntaxFormFieldKind::Tags,
            value: "#food #".to_string(),
            placeholder: String::new(),
            multiline: false,
            required: false,
            satisfied: true,
            focused: true,
            suggestion_query: "#".to_string(),
            selected_suggestion_index: Some(0),
            suggestions: vec![],
        };
        let suggestion = MenuSyntaxFormSuggestion {
            value: "#food".to_string(),
            label: "#food (3)".to_string(),
            source: "tagHistory".to_string(),
            detail: Some("Recent tag".to_string()),
        };
        let application =
            apply_menu_syntax_form_suggestion(&field, &suggestion).expect("apply suggestion");
        let rewritten =
            apply_capture_form_field_edit(&invocation, "tags", &application.next_field_value)
                .expect("tags edit should serialize");

        assert_eq!(rewritten, ";todo Eat lunch #food");
    }

    #[test]
    fn priority_suggestions_expose_full_schema_choice_set() {
        let schema = builtin_schema("todo").expect("todo schema");
        let invocation = todo_invocation(";todo Renew passport p2");
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            2,
            &validation,
            MenuSyntaxFormSuggestionPools {
                priority_values: vec![
                    "p1".to_string(),
                    "p2".to_string(),
                    "p3".to_string(),
                    "p4".to_string(),
                ],
                ..Default::default()
            },
        );
        let priority = snapshot
            .fields
            .iter()
            .find(|field| field.id == "priority")
            .expect("priority field");

        assert_eq!(
            priority
                .suggestions
                .iter()
                .map(|suggestion| suggestion.value.as_str())
                .collect::<Vec<_>>(),
            vec!["p1", "p2", "p3", "p4"]
        );
    }

    #[test]
    fn accepting_priority_suggestion_rewrites_canonical_capture_text() {
        let invocation = todo_invocation(";todo Renew passport p1 #errands");
        let field = MenuSyntaxFormFieldSnapshot {
            id: "priority".to_string(),
            label: "Priority".to_string(),
            kind: MenuSyntaxFormFieldKind::Priority,
            value: "p1".to_string(),
            placeholder: String::new(),
            multiline: false,
            required: false,
            satisfied: true,
            focused: true,
            suggestion_query: "p1".to_string(),
            selected_suggestion_index: Some(1),
            suggestions: vec![],
        };
        let suggestion = MenuSyntaxFormSuggestion {
            value: "p2".to_string(),
            label: "p2".to_string(),
            source: "schema".to_string(),
            detail: None,
        };
        let application =
            apply_menu_syntax_form_suggestion(&field, &suggestion).expect("apply suggestion");
        let rewritten =
            apply_capture_form_field_edit(&invocation, "priority", &application.next_field_value)
                .expect("priority edit should serialize");

        assert_eq!(application.next_field_value, "p2");
        assert_eq!(rewritten, ";todo Renew passport #errands p2");
    }

    #[test]
    fn invalid_priority_field_value_is_rejected() {
        let invocation = todo_invocation(";todo Renew passport p1");

        assert!(apply_capture_form_field_edit(&invocation, "priority", "urgent").is_none());
    }

    #[test]
    fn object_field_reflects_existing_body_object_token() {
        let schema = builtin_schema("todo").expect("todo schema");
        let invocation = todo_invocation(";todo Renew passport @todo:todo_1 #errands");
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            5,
            &validation,
            MenuSyntaxFormSuggestionPools::default(),
        );
        let object = snapshot
            .fields
            .iter()
            .find(|field| field.id == "object")
            .expect("object field");

        assert_eq!(object.value, "@todo:todo_1");
    }

    #[test]
    fn clearing_object_field_removes_object_token_from_body() {
        let invocation = todo_invocation(";todo Renew passport @todo:todo_1 #errands");
        let rewritten = apply_capture_form_field_edit(&invocation, "object", "")
            .expect("object edit should serialize");

        assert_eq!(rewritten, ";todo Renew passport #errands");
    }

    #[test]
    fn object_suggestions_filter_existing_candidates() {
        let schema = builtin_schema("todo").expect("todo schema");
        let invocation = todo_invocation(";todo Renew passport @wel");
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            5,
            &validation,
            MenuSyntaxFormSuggestionPools {
                objects: vec![
                    crate::menu_syntax::ObjectSelectorCandidate {
                        kind: crate::menu_syntax::CaptureObjectKind::Todo,
                        id: "todo_1".to_string(),
                        label: "Welcome to Todo App".to_string(),
                        subtitle: "Open".to_string(),
                    },
                    crate::menu_syntax::ObjectSelectorCandidate {
                        kind: crate::menu_syntax::CaptureObjectKind::Todo,
                        id: "todo_2".to_string(),
                        label: "Review passport".to_string(),
                        subtitle: "Open".to_string(),
                    },
                ],
                ..Default::default()
            },
        );
        let object = snapshot
            .fields
            .iter()
            .find(|field| field.id == "object")
            .expect("object field");

        assert_eq!(object.suggestion_query, "@wel");
        assert_eq!(object.suggestions.len(), 1);
        assert_eq!(object.suggestions[0].value, "@todo:todo_1");
    }

    #[test]
    fn bare_object_query_stays_out_of_body_while_form_edits() {
        let invocation = todo_invocation(";todo Buy milk @ #errands");

        assert_eq!(invocation.body, "Buy milk");
        assert_eq!(invocation.tags, vec!["errands".to_string()]);
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

    #[test]
    fn snippet_form_labels_body_as_snippet_and_exposes_metadata_fields() {
        let schema = builtin_schema("snippet").expect("snippet schema");
        let invocation = todo_invocation(
            ";snippet Hello there! keyword:hi! description:Expand hi! to hello! name:Hi to Hello",
        );
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            0,
            &validation,
            MenuSyntaxFormSuggestionPools::default(),
        );

        let body = snapshot
            .fields
            .iter()
            .find(|field| field.id == "body")
            .expect("body field");
        assert_eq!(body.label, "Snippet");
        assert_eq!(body.placeholder, "Text to paste/expand");
        assert!(body.multiline);
        assert!(snapshot
            .fields
            .iter()
            .any(|field| field.id == "kv:name" && field.required));
        assert!(snapshot.fields.iter().any(|field| field.id == "kv:keyword"));
        assert!(snapshot
            .fields
            .iter()
            .any(|field| field.id == "kv:description"));
        assert!(!snapshot.fields.iter().any(|field| field.id == "trigger"));
        assert!(snapshot.can_submit);
    }

    #[test]
    fn snippet_form_body_is_multiline_and_placeholder_clean() {
        let schema = builtin_schema("snippet").expect("snippet schema");
        let invocation = todo_invocation(
            ";snippet Hello there! keyword:hi name:Hi description:Expands greeting",
        );
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            0,
            &validation,
            MenuSyntaxFormSuggestionPools::default(),
        );

        let body = snapshot
            .fields
            .iter()
            .find(|field| field.id == "body")
            .expect("body field");
        assert_eq!(body.kind, MenuSyntaxFormFieldKind::Body);
        assert!(body.multiline);
        assert_eq!(body.placeholder, "Text to paste/expand");

        let name = snapshot
            .fields
            .iter()
            .find(|field| field.id == "kv:name")
            .expect("name field");
        assert_eq!(name.placeholder, "Snippet name");
        let keyword = snapshot
            .fields
            .iter()
            .find(|field| field.id == "kv:keyword")
            .expect("keyword field");
        assert_eq!(keyword.placeholder, "Expansion keyword");
        let description = snapshot
            .fields
            .iter()
            .find(|field| field.id == "kv:description")
            .expect("description field");
        assert_eq!(description.placeholder, "Optional description");

        for field in snapshot
            .fields
            .iter()
            .filter(|field| field.id.starts_with("kv:"))
        {
            let key = field.id.trim_start_matches("kv:");
            assert!(
                !field.placeholder.starts_with(&format!("{key}:")),
                "{} placeholder should not duplicate its key prefix",
                field.id
            );
            assert_ne!(field.placeholder, format!("{key}{}", ":value"));
        }
    }

    #[test]
    fn todo_body_remains_single_line() {
        let schema = builtin_schema("todo").expect("todo schema");
        let invocation = todo_invocation(";todo Renew passport");
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            0,
            &validation,
            MenuSyntaxFormSuggestionPools::default(),
        );

        let body = snapshot
            .fields
            .iter()
            .find(|field| field.id == "body")
            .expect("body field");
        assert_eq!(body.kind, MenuSyntaxFormFieldKind::Body);
        assert!(!body.multiline);
    }

    #[test]
    fn editing_snippet_body_preserves_internal_newlines() {
        let invocation = todo_invocation(";snippet add name:\"Email reply\" keyword:reply");
        let body = "Hello John,\n\nThanks for the update.\nBest,\nMe";
        let rewritten = apply_capture_form_field_edit(&invocation, "body", body)
            .expect("body edit should serialize");

        assert!(rewritten.contains(" -- "));
        let reparsed = todo_invocation(&rewritten);
        let draft = parse_snippet_scriptlet_capture(&reparsed).expect("snippet draft");
        assert_eq!(draft.operation, SnippetScriptletOperation::Create);
        assert_eq!(draft.body.as_deref(), Some(body));
        assert_eq!(draft.name.as_deref(), Some("Email reply"));
        assert_eq!(draft.keyword.as_deref(), Some("reply"));
    }

    #[test]
    fn editing_snippet_metadata_keeps_body_after_delimiter() {
        let invocation = todo_invocation(";snippet add keyword:reply");
        let body = "Hello,\n\nThis body mentions keyword:inside.\nBye";
        let with_body = apply_capture_form_field_edit(&invocation, "body", body)
            .expect("body edit should serialize");
        let with_name =
            apply_capture_form_field_edit(&todo_invocation(&with_body), "kv:name", "Email body")
                .expect("name edit should serialize");

        assert!(with_name.contains(" -- "));
        let reparsed = todo_invocation(&with_name);
        let draft = parse_snippet_scriptlet_capture(&reparsed).expect("snippet draft");
        assert_eq!(draft.body.as_deref(), Some(body));
        assert_eq!(draft.name.as_deref(), Some("Email body"));
        assert_eq!(draft.keyword.as_deref(), Some("reply"));
    }

    #[test]
    fn link_form_labels_body_as_title_and_exposes_metadata_fields() {
        let schema = builtin_schema("link").expect("link schema");
        let invocation = todo_invocation(";link https://example.com Example description:Docs");
        let validation = validate(&invocation, &schema);
        let snapshot = build_capture_form_snapshot(
            &schema,
            &invocation,
            0,
            &validation,
            MenuSyntaxFormSuggestionPools::default(),
        );

        let body = snapshot
            .fields
            .iter()
            .find(|field| field.id == "body")
            .expect("body field");
        assert_eq!(body.label, "Title");
        assert_eq!(body.placeholder, "Optional link title");
        assert!(snapshot
            .fields
            .iter()
            .any(|field| field.id == "url" && field.required));
        assert!(!snapshot.fields.iter().any(|field| field.id == "kv:title"));
        assert!(snapshot
            .fields
            .iter()
            .any(|field| field.id == "kv:description"));
        assert!(snapshot.fields.iter().any(|field| field.id == "tags"));
        assert!(!snapshot.fields.iter().any(|field| field.id == "kv:tags"));
        assert!(snapshot.can_submit);
    }
}
