use serde_json::{Map, Value};

use super::payload::{
    object_refs_for_raw_capture, CaptureInvocation, CaptureObjectKind, CaptureObjectRef,
};
use super::snippet_scriptlet::{
    ActiveCaptureFieldSelector, SnippetMetadataFieldSpec, SnippetMetadataValueKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkScriptletOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkLookup {
    SelectedRef(String),
    Url(String),
    Title(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkScriptletDraft {
    pub operation: LinkScriptletOperation,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub metadata: Map<String, Value>,
    pub lookup: Option<LinkLookup>,
    pub object_refs: Vec<CaptureObjectRef>,
    pub raw: String,
}

const FIELD_SPECS: &[SnippetMetadataFieldSpec] = &[
    SnippetMetadataFieldSpec {
        key: "url",
        aliases: &["href", "link"],
        label: "url",
        required_on_create: true,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "title",
        aliases: &["name", "label"],
        label: "title",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "description",
        aliases: &["desc"],
        label: "description",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "alias",
        aliases: &[],
        label: "alias",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "shortcut",
        aliases: &["hotkey"],
        label: "shortcut",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "author",
        aliases: &[],
        label: "author",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "icon",
        aliases: &[],
        label: "icon",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "hidden",
        aliases: &[],
        label: "hidden",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::Boolean,
    },
    SnippetMetadataFieldSpec {
        key: "background",
        aliases: &[],
        label: "background",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::Boolean,
    },
    SnippetMetadataFieldSpec {
        key: "system",
        aliases: &[],
        label: "system",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::Boolean,
    },
    SnippetMetadataFieldSpec {
        key: "tags",
        aliases: &["tag"],
        label: "tags",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::List,
    },
    SnippetMetadataFieldSpec {
        key: "watch",
        aliases: &[],
        label: "watch",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::List,
    },
];

pub fn link_metadata_field_specs() -> &'static [SnippetMetadataFieldSpec] {
    FIELD_SPECS
}

pub fn canonical_link_metadata_key(key: &str) -> Option<&'static str> {
    let key = key.trim().to_ascii_lowercase();
    FIELD_SPECS.iter().find_map(|spec| {
        (spec.key == key || spec.aliases.iter().any(|alias| *alias == key)).then_some(spec.key)
    })
}

pub fn normalize_link_capture_invocation(invocation: &mut CaptureInvocation) {
    if !invocation.target.eq_ignore_ascii_case("link") {
        return;
    }
    if let Ok(draft) = parse_link_scriptlet_capture(invocation) {
        invocation.url = draft.url.clone().or_else(|| invocation.url.clone());
        invocation.body = draft.title.clone().unwrap_or_default();
        invocation.kv = draft
            .metadata
            .iter()
            .filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_string())))
            .collect();
        if let Some(tags) = draft.metadata.get("tags").and_then(Value::as_array) {
            for tag in tags.iter().filter_map(Value::as_str) {
                if !invocation
                    .tags
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(tag))
                {
                    invocation.tags.push(tag.to_string());
                }
            }
        }
    }
}

pub fn parse_link_scriptlet_capture(
    invocation: &CaptureInvocation,
) -> Result<LinkScriptletDraft, String> {
    if !invocation.target.eq_ignore_ascii_case("link") {
        return Err("Not a link capture.".to_string());
    }
    let rest = capture_rest_from_raw(&invocation.raw).unwrap_or_else(|| invocation.body.clone());
    let (control, body_after_delimiter) = split_double_dash(&rest);
    let tokens = tokenize(control);
    let mut operation = LinkScriptletOperation::Create;
    let mut body_tokens: Vec<String> = Vec::new();
    let mut metadata = Map::new();
    let mut index = 0usize;
    let mut metadata_started = false;

    while index < tokens.len() {
        let token = &tokens[index];
        if index == 0 {
            if let Some(op) = operation_from_word(token) {
                operation = op;
                index += 1;
                continue;
            }
        }
        if token.starts_with('@') {
            index += 1;
            continue;
        }
        if let Some((key, inline)) = split_metadata_token(token) {
            if let Some(canonical) = canonical_link_metadata_key(key) {
                metadata_started = true;
                let mut value_parts = Vec::new();
                if !inline.is_empty() {
                    value_parts.push(inline);
                }
                index += 1;
                while index < tokens.len() {
                    if tokens[index].starts_with('@') || tokens[index].starts_with('#') {
                        index += 1;
                        continue;
                    }
                    if split_metadata_token(&tokens[index])
                        .and_then(|(next_key, _)| canonical_link_metadata_key(next_key))
                        .is_some()
                    {
                        break;
                    }
                    value_parts.push(tokens[index].clone());
                    index += 1;
                }
                let value = value_parts.join(" ").trim().to_string();
                if !value.is_empty() {
                    insert_metadata_value(&mut metadata, canonical, &value);
                }
                continue;
            }
        }
        if !metadata_started {
            body_tokens.push(token.clone());
        }
        index += 1;
    }

    for tag in &invocation.tags {
        insert_list_metadata_value(&mut metadata, "tags", tag);
    }

    let body = body_after_delimiter
        .map(|body| body.trim().to_string())
        .filter(|body| !body.is_empty())
        .or_else(|| {
            let body = body_tokens.join(" ").trim().to_string();
            (!body.is_empty()).then_some(body)
        });

    let object_refs = object_refs_for_raw_capture(&invocation.target, &invocation.raw);
    let selected_ref = object_refs.iter().find(|object_ref| {
        object_ref.resolved
            && object_ref.role == "primary"
            && object_ref.kind == CaptureObjectKind::Link
    });
    let selected_url = selected_ref.map(|object_ref| object_ref.id.clone());
    let metadata_url = metadata
        .get("url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let body_url = body.as_deref().and_then(first_http_url);
    let explicit_url = metadata_url.or_else(|| invocation.url.clone()).or(body_url);
    if let (Some(explicit_url), Some(selected_url)) =
        (explicit_url.as_deref(), selected_url.as_deref())
    {
        if explicit_url != selected_url {
            return Err("Selected link does not match the explicit URL.".to_string());
        }
    }
    let url = explicit_url.or_else(|| {
        matches!(
            operation,
            LinkScriptletOperation::Update | LinkScriptletOperation::Delete
        )
        .then(|| selected_url)
        .flatten()
    });
    if let Some(url) = url.as_deref() {
        if !is_http_url(url) {
            return Err(format!(
                "URL must start with http:// or https://, got `{url}`"
            ));
        }
        metadata.insert("url".to_string(), Value::String(url.to_string()));
    }

    let title = metadata
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| title_from_body(body.as_deref(), url.as_deref()));
    if let Some(title) = title.as_deref() {
        metadata.insert("title".to_string(), Value::String(title.to_string()));
    }
    metadata.insert("tool".to_string(), Value::String("open".to_string()));
    let description = metadata
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let lookup = selected_ref
        .map(|object_ref| LinkLookup::SelectedRef(object_ref.id.clone()))
        .or_else(|| url.clone().map(LinkLookup::Url))
        .or_else(|| title.clone().map(LinkLookup::Title));

    Ok(LinkScriptletDraft {
        operation,
        url,
        title,
        description,
        metadata,
        lookup,
        object_refs,
        raw: invocation.raw.clone(),
    })
}

pub fn active_link_capture_field_selector_for_input(
    input: &str,
) -> Option<ActiveCaptureFieldSelector> {
    let (target, body_start) = capture_target_and_body_start(input)?;
    if !target.eq_ignore_ascii_case("link") {
        return None;
    }
    let body = &input[body_start..];
    let delimiter_start = find_double_dash_token(body).unwrap_or(body.len());
    let before_delimiter = &body[..delimiter_start];
    let cursor = before_delimiter.len();
    let token_start = before_delimiter[..cursor]
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_whitespace().then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    let token = &before_delimiter[token_start..cursor];
    let query = if let Some(query) = token.strip_prefix(':') {
        query
    } else if token.ends_with(':') && token.matches(':').count() == 1 {
        token.trim_end_matches(':')
    } else {
        return None;
    };
    if query.contains(':') || query.contains('=') {
        return None;
    }
    let fields = FIELD_SPECS
        .iter()
        .copied()
        .filter(|spec| {
            query.is_empty()
                || spec.key.starts_with(&query.to_ascii_lowercase())
                || spec
                    .aliases
                    .iter()
                    .any(|alias| alias.starts_with(&query.to_ascii_lowercase()))
        })
        .collect::<Vec<_>>();
    if fields.is_empty() {
        return None;
    }
    Some(ActiveCaptureFieldSelector {
        target,
        query: query.to_string(),
        range: (body_start + token_start, body_start + cursor),
        fields,
    })
}

fn insert_metadata_value(metadata: &mut Map<String, Value>, key: &str, raw_value: &str) {
    let value = match FIELD_SPECS
        .iter()
        .find(|spec| spec.key == key)
        .map(|spec| spec.value_kind)
        .unwrap_or(SnippetMetadataValueKind::String)
    {
        SnippetMetadataValueKind::Boolean => Value::Bool(matches!(
            raw_value.trim().to_ascii_lowercase().as_str(),
            "true" | "yes" | "1" | "on"
        )),
        SnippetMetadataValueKind::List => Value::Array(
            raw_value
                .split(',')
                .flat_map(|part| part.split_whitespace())
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(|part| Value::String(part.trim_start_matches('#').to_string()))
                .collect(),
        ),
        SnippetMetadataValueKind::String => Value::String(unquote(raw_value)),
    };
    metadata.insert(key.to_string(), value);
}

fn insert_list_metadata_value(metadata: &mut Map<String, Value>, key: &str, raw_value: &str) {
    let entry = metadata
        .entry(key.to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    if let Value::Array(values) = entry {
        let value = raw_value.trim().trim_start_matches('#');
        if !value.is_empty()
            && !values
                .iter()
                .any(|existing| existing.as_str().map(|s| s == value).unwrap_or(false))
        {
            values.push(Value::String(value.to_string()));
        }
    }
}

fn capture_rest_from_raw(raw: &str) -> Option<String> {
    let (_, body_start) = capture_target_and_body_start(raw)?;
    Some(raw[body_start..].trim_start().to_string())
}

fn capture_target_and_body_start(input: &str) -> Option<(String, usize)> {
    if let Some(rest) = input.strip_prefix(';').or_else(|| input.strip_prefix('+')) {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        if end == 0 {
            return None;
        }
        return Some((rest[..end].to_ascii_lowercase(), 1 + end));
    }
    let colon_idx = input.find(':')?;
    let head = &input[..colon_idx];
    if head.is_empty() || head.contains(char::is_whitespace) {
        return None;
    }
    Some((head.to_ascii_lowercase(), colon_idx + 1))
}

fn split_double_dash(input: &str) -> (&str, Option<&str>) {
    if let Some(idx) = find_double_dash_token(input) {
        (&input[..idx], Some(input[idx + 2..].trim_start()))
    } else {
        (input, None)
    }
}

fn find_double_dash_token(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'-'
            && bytes[i + 1] == b'-'
            && (i == 0 || bytes[i - 1].is_ascii_whitespace())
            && (i + 2 == bytes.len() || bytes[i + 2].is_ascii_whitespace())
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn split_metadata_token(token: &str) -> Option<(&str, String)> {
    let idx = token.find(':').or_else(|| token.find('='))?;
    let key = &token[..idx];
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return None;
    }
    Some((key, token[idx + 1..].to_string()))
}

fn operation_from_word(word: &str) -> Option<LinkScriptletOperation> {
    match word.to_ascii_lowercase().as_str() {
        "add" | "create" | "save" => Some(LinkScriptletOperation::Create),
        "update" => Some(LinkScriptletOperation::Update),
        "remove" | "rm" | "delete" => Some(LinkScriptletOperation::Delete),
        _ => None,
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        let mut quote: Option<u8> = None;
        while i < bytes.len() {
            let c = bytes[i];
            match quote {
                Some(q) if c == q => {
                    quote = None;
                    i += 1;
                }
                Some(_) => i += 1,
                None if c == b'"' || c == b'\'' => {
                    quote = Some(c);
                    i += 1;
                }
                None if c.is_ascii_whitespace() => break,
                None => i += 1,
            }
        }
        out.push(unquote(&input[start..i]));
    }
    out
}

fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn first_http_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|part| is_http_url(part))
        .map(ToString::to_string)
}

fn title_from_body(body: Option<&str>, url: Option<&str>) -> Option<String> {
    let body = body?;
    let title = body
        .split_whitespace()
        .filter(|part| !part.starts_with('@'))
        .filter(|part| url.map(|url| *part != url).unwrap_or(true))
        .filter(|part| !is_http_url(part))
        .collect::<Vec<_>>()
        .join(" ");
    (!title.trim().is_empty()).then(|| title.trim().to_string())
}

pub(crate) fn is_http_url(text: &str) -> bool {
    text.starts_with("http://") || text.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};

    fn draft(input: &str) -> LinkScriptletDraft {
        let invocation = match parse_capture(input) {
            CaptureParse::Ok(invocation) => invocation,
            CaptureParse::Incomplete(incomplete) => panic!("{incomplete:?}"),
        };
        parse_link_scriptlet_capture(&invocation).expect("link draft")
    }

    #[test]
    fn parses_url_title_and_unquoted_link_metadata() {
        let draft = draft(";link https://example.com Example description:Docs #docs");

        assert_eq!(draft.operation, LinkScriptletOperation::Create);
        assert_eq!(draft.url.as_deref(), Some("https://example.com"));
        assert_eq!(draft.title.as_deref(), Some("Example"));
        assert_eq!(draft.description.as_deref(), Some("Docs"));
        assert_eq!(draft.metadata["tags"][0], Value::String("docs".to_string()));
    }

    #[test]
    fn normalizes_href_link_name_label_aliases() {
        let draft = draft(";link href:https://example.com name:Example");
        assert_eq!(draft.url.as_deref(), Some("https://example.com"));
        assert_eq!(draft.title.as_deref(), Some("Example"));
        assert!(draft.metadata.get("href").is_none());
        assert!(draft.metadata.get("name").is_none());
    }

    #[test]
    fn update_selected_ref_uses_selected_url_when_url_is_missing() {
        let draft = draft(";link update @link:https://example.com title:New");
        assert_eq!(draft.operation, LinkScriptletOperation::Update);
        assert_eq!(draft.url.as_deref(), Some("https://example.com"));
        assert_eq!(draft.title.as_deref(), Some("New"));
        assert_eq!(
            draft.lookup,
            Some(LinkLookup::SelectedRef("https://example.com".to_string()))
        );
    }

    #[test]
    fn delete_selected_ref_has_no_title_requirement() {
        let draft = draft(";link delete @link:https://example.com");
        assert_eq!(draft.operation, LinkScriptletOperation::Delete);
        assert_eq!(draft.url.as_deref(), Some("https://example.com"));
        assert_eq!(draft.title, None);
    }

    #[test]
    fn selected_ref_explicit_url_mismatch_rejects() {
        let invocation =
            match parse_capture(";link update @link:https://a.example url:https://b.example") {
                CaptureParse::Ok(invocation) => invocation,
                CaptureParse::Incomplete(incomplete) => panic!("{incomplete:?}"),
            };
        let err = parse_link_scriptlet_capture(&invocation).expect_err("mismatch");
        assert_eq!(err, "Selected link does not match the explicit URL.");
    }

    #[test]
    fn active_field_selector_detects_link_colon_field_query() {
        let selector =
            active_link_capture_field_selector_for_input(";link https://example.com :ti")
                .expect("field selector");
        assert_eq!(selector.query, "ti");
        assert_eq!(selector.fields[0].key, "title");

        let selector =
            active_link_capture_field_selector_for_input(";link https://example.com ti:")
                .expect("field selector");
        assert_eq!(selector.fields[0].key, "title");
    }
}
