use serde_json::{Map, Value};

use super::payload::{
    object_refs_for_raw_capture, CaptureInvocation, CaptureObjectKind, CaptureObjectRef,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnippetScriptletOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnippetLookup {
    SelectedRef(String),
    Keyword(String),
    Name(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetMetadataValueKind {
    String,
    Boolean,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnippetMetadataFieldSpec {
    pub key: &'static str,
    pub aliases: &'static [&'static str],
    pub label: &'static str,
    pub required_on_create: bool,
    pub value_kind: SnippetMetadataValueKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnippetScriptletDraft {
    pub operation: SnippetScriptletOperation,
    pub body: Option<String>,
    pub name: Option<String>,
    pub keyword: Option<String>,
    pub description: Option<String>,
    pub metadata: Map<String, Value>,
    pub lookup: Option<SnippetLookup>,
    pub object_refs: Vec<CaptureObjectRef>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveCaptureFieldSelector {
    pub target: String,
    pub query: String,
    pub range: (usize, usize),
    pub fields: Vec<SnippetMetadataFieldSpec>,
}

const FIELD_SPECS: &[SnippetMetadataFieldSpec] = &[
    SnippetMetadataFieldSpec {
        key: "name",
        aliases: &[],
        label: "name",
        required_on_create: true,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "keyword",
        aliases: &["trigger", "expand", "snippet"],
        label: "keyword",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "description",
        aliases: &[],
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
        aliases: &[],
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
        key: "enter",
        aliases: &[],
        label: "enter",
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
        key: "placeholder",
        aliases: &[],
        label: "placeholder",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "cron",
        aliases: &[],
        label: "cron",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "schedule",
        aliases: &[],
        label: "schedule",
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
        key: "fallback",
        aliases: &[],
        label: "fallback",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::Boolean,
    },
    SnippetMetadataFieldSpec {
        key: "fallback_label",
        aliases: &[],
        label: "fallback_label",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
    SnippetMetadataFieldSpec {
        key: "tags",
        aliases: &[],
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
    SnippetMetadataFieldSpec {
        key: "tool",
        aliases: &["lang", "language"],
        label: "tool",
        required_on_create: false,
        value_kind: SnippetMetadataValueKind::String,
    },
];

pub fn snippet_metadata_field_specs() -> &'static [SnippetMetadataFieldSpec] {
    FIELD_SPECS
}

pub fn canonical_snippet_metadata_key(key: &str) -> Option<&'static str> {
    let key = key.trim().to_ascii_lowercase();
    FIELD_SPECS.iter().find_map(|spec| {
        (spec.key == key || spec.aliases.iter().any(|alias| *alias == key)).then_some(spec.key)
    })
}

pub fn normalize_snippet_capture_invocation(invocation: &mut CaptureInvocation) {
    if !invocation.target.eq_ignore_ascii_case("snippet") {
        return;
    }
    if let Ok(draft) = parse_snippet_scriptlet_capture(invocation) {
        invocation.body = draft.body.unwrap_or_default();
        invocation.kv = draft
            .metadata
            .iter()
            .filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_string())))
            .collect();
    }
}

pub fn parse_snippet_scriptlet_capture(
    invocation: &CaptureInvocation,
) -> Result<SnippetScriptletDraft, String> {
    if !invocation.target.eq_ignore_ascii_case("snippet") {
        return Err("Not a snippet capture.".to_string());
    }
    let rest = capture_rest_from_raw(&invocation.raw).unwrap_or_else(|| invocation.body.clone());
    let (control, body_after_delimiter) = split_double_dash(&rest);
    let tokens = tokenize(control);
    let mut operation = SnippetScriptletOperation::Create;
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
            if let Some(canonical) = canonical_snippet_metadata_key(key) {
                metadata_started = true;
                let mut value_parts = Vec::new();
                if !inline.is_empty() {
                    value_parts.push(inline);
                }
                index += 1;
                while index < tokens.len() {
                    if tokens[index].starts_with('@') {
                        index += 1;
                        continue;
                    }
                    if split_metadata_token(&tokens[index])
                        .and_then(|(next_key, _)| canonical_snippet_metadata_key(next_key))
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
            && object_ref.kind == CaptureObjectKind::Snippet
    });
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let keyword = metadata
        .get("keyword")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let description = metadata
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let lookup = selected_ref
        .map(|object_ref| SnippetLookup::SelectedRef(object_ref.id.clone()))
        .or_else(|| keyword.clone().map(SnippetLookup::Keyword))
        .or_else(|| name.clone().map(SnippetLookup::Name));

    Ok(SnippetScriptletDraft {
        operation,
        body,
        name,
        keyword,
        description,
        metadata,
        lookup,
        object_refs,
        raw: invocation.raw.clone(),
    })
}

pub fn active_capture_field_selector_for_input(
    input: &str,
    registered_targets: &[String],
) -> Option<ActiveCaptureFieldSelector> {
    let (target, body_start) = capture_target_and_body_start(input)?;
    if !target.eq_ignore_ascii_case("snippet")
        && !registered_targets
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&target))
    {
        return None;
    }
    if !target.eq_ignore_ascii_case("snippet") {
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

fn operation_from_word(word: &str) -> Option<SnippetScriptletOperation> {
    match word.to_ascii_lowercase().as_str() {
        "add" | "create" | "save" => Some(SnippetScriptletOperation::Create),
        "update" => Some(SnippetScriptletOperation::Update),
        "remove" | "rm" | "delete" => Some(SnippetScriptletOperation::Delete),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};

    fn draft(input: &str) -> SnippetScriptletDraft {
        let invocation = match parse_capture(input) {
            CaptureParse::Ok(invocation) => invocation,
            CaptureParse::Incomplete(incomplete) => panic!("{incomplete:?}"),
        };
        parse_snippet_scriptlet_capture(&invocation).expect("snippet draft")
    }

    #[test]
    fn parses_unquoted_multi_word_snippet_metadata() {
        let draft = draft(
            ";snippet Hello there! keyword:hi! description:Expand hi! to hello! name:Hi to Hello",
        );

        assert_eq!(draft.operation, SnippetScriptletOperation::Create);
        assert_eq!(draft.body.as_deref(), Some("Hello there!"));
        assert_eq!(draft.keyword.as_deref(), Some("hi!"));
        assert_eq!(draft.description.as_deref(), Some("Expand hi! to hello!"));
        assert_eq!(draft.name.as_deref(), Some("Hi to Hello"));
    }

    #[test]
    fn preserves_body_after_double_dash_even_with_keyword_like_text() {
        let draft = draft(";snippet add trigger:fj name:Fetch JSON -- const keyword:value = 1");

        assert_eq!(draft.body.as_deref(), Some("const keyword:value = 1"));
        assert_eq!(draft.keyword.as_deref(), Some("fj"));
        assert_eq!(draft.name.as_deref(), Some("Fetch JSON"));
    }

    #[test]
    fn normalizes_trigger_expand_snippet_aliases_to_keyword() {
        for alias in ["trigger", "expand", "snippet"] {
            let draft = draft(&format!(";snippet Body {alias}:hi name:Hi"));
            assert_eq!(draft.keyword.as_deref(), Some("hi"));
            assert!(draft.metadata.get(alias).is_none());
        }
    }

    #[test]
    fn normalizes_lang_language_to_tool() {
        for alias in ["lang", "language"] {
            let draft = draft(&format!(";snippet Body {alias}:paste name:Hi"));
            assert_eq!(
                draft.metadata.get("tool").and_then(Value::as_str),
                Some("paste")
            );
        }
    }

    #[test]
    fn delete_selected_ref_requires_no_body() {
        let draft = draft(";snippet delete @snippet:hi");

        assert_eq!(draft.operation, SnippetScriptletOperation::Delete);
        assert_eq!(draft.body, None);
        assert_eq!(
            draft.lookup,
            Some(SnippetLookup::SelectedRef("hi".to_string()))
        );
    }

    #[test]
    fn update_selected_ref_preserves_missing_body() {
        let draft = draft(";snippet update @snippet:hi description:New desc");

        assert_eq!(draft.operation, SnippetScriptletOperation::Update);
        assert_eq!(draft.body, None);
        assert_eq!(draft.description.as_deref(), Some("New desc"));
    }

    #[test]
    fn active_field_selector_detects_snippet_colon_field_query() {
        let selector = active_capture_field_selector_for_input(";snippet Hello there! de", &[]);
        assert!(selector.is_none());

        let selector = active_capture_field_selector_for_input(";snippet Hello there! :de", &[])
            .expect("field selector");
        assert_eq!(selector.query, "de");
        assert_eq!(selector.fields[0].key, "description");

        let selector = active_capture_field_selector_for_input(";snippet Hello there! de:", &[])
            .expect("field selector");
        assert_eq!(selector.query, "de");
        assert_eq!(selector.fields[0].key, "description");
    }
}
