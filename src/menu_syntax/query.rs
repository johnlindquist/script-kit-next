use super::payload::{
    source_for_head, AdvancedQuery, ArtifactKind, Predicate, RootUnifiedSourceFilter,
    RootUnifiedSourceFilterSet, ShortcutPredicate,
};

pub fn parse_advanced_query(input: &str) -> AdvancedQuery {
    let raw = input.to_string();
    let stripped = input.strip_prefix(':').unwrap_or(input);
    let tokens = tokenize(stripped);

    let mut predicates: Vec<Predicate> = Vec::new();
    let mut free_parts: Vec<String> = Vec::new();

    for token in tokens {
        let (negated, body) = if let Some(rest) = token.strip_prefix('-') {
            if rest.contains(':') || rest.starts_with('#') {
                (true, rest.to_string())
            } else {
                (false, token)
            }
        } else {
            (false, token)
        };

        match classify_qualifier(&body) {
            Some(pred) => {
                if negated {
                    predicates.push(Predicate::Negate(Box::new(pred)));
                } else {
                    predicates.push(pred);
                }
            }
            None => free_parts.push(body),
        }
    }

    AdvancedQuery {
        free_text: free_parts.join(" ").trim().to_string(),
        predicates,
        source_filters: RootUnifiedSourceFilterSet::default(),
        raw,
    }
}

pub fn parse_source_filter_query(input: &str) -> Option<AdvancedQuery> {
    parse_filter_query(input)
}

pub fn parse_filter_query(input: &str) -> Option<AdvancedQuery> {
    let tokens = tokenize(input);
    if tokens.is_empty() {
        return None;
    }

    if input.starts_with(':') {
        return None;
    }

    let mut source_filters = RootUnifiedSourceFilterSet::default();
    let mut predicates: Vec<Predicate> = Vec::new();
    let mut free_parts = Vec::new();
    let mut claimed_filter = false;

    for token in tokens {
        if let Some((negated, source)) = classify_source_filter_token(&token) {
            if negated {
                source_filters.exclude(source);
            } else {
                source_filters.insert(source);
            }
            claimed_filter = true;
        } else if let Some((negated, pred)) = classify_filter_predicate_token(&token) {
            if negated {
                predicates.push(Predicate::Negate(Box::new(pred)));
            } else {
                predicates.push(pred);
            }
            claimed_filter = true;
        } else {
            free_parts.push(token);
        }
    }

    if !claimed_filter {
        return None;
    }

    Some(AdvancedQuery {
        free_text: free_parts.join(" ").trim().to_string(),
        predicates,
        source_filters,
        raw: input.to_string(),
    })
}

fn classify_source_filter_token(token: &str) -> Option<(bool, RootUnifiedSourceFilter)> {
    if quoted(token) {
        return None;
    }
    let (negated, body) = token
        .strip_prefix('-')
        .map(|rest| (true, rest))
        .unwrap_or((false, token));
    source_for_head(body).map(|source| (negated, source))
}

fn classify_filter_predicate_token(token: &str) -> Option<(bool, Predicate)> {
    if quoted(token) {
        return None;
    }
    let (negated, body) = token
        .strip_prefix('-')
        .map(|rest| (true, rest))
        .unwrap_or((false, token));
    classify_qualifier(body).map(|pred| (negated, pred))
}

fn classify_qualifier(token: &str) -> Option<Predicate> {
    if let Some(tag) = token.strip_prefix('#') {
        if !tag.is_empty() {
            return Some(Predicate::Tag(tag.to_string()));
        }
        return None;
    }

    let (key_raw, value_raw) = token.split_once(':')?;
    let key = key_raw.trim();
    let value = unquote(value_raw.trim());

    if key.is_empty() {
        return None;
    }

    if let Some(path) = key.strip_prefix("meta.") {
        if path.is_empty() || value.is_empty() {
            return None;
        }
        return Some(Predicate::MetaPath {
            path: path.to_string(),
            value,
        });
    }

    match key {
        "type" | "kind" => ArtifactKind::parse(&value).map(Predicate::Type),
        "shortcut" => Some(Predicate::HasShortcut(
            match value.to_ascii_lowercase().as_str() {
                "true" | "yes" | "1" | "any" => ShortcutPredicate::Any,
                "false" | "no" | "0" | "none" => ShortcutPredicate::None,
                _ if value.is_empty() => ShortcutPredicate::Any,
                _ => ShortcutPredicate::Literal(value.clone()),
            },
        )),
        "source" => (!value.is_empty()).then_some(Predicate::Source(value)),
        "plugin" => (!value.is_empty()).then_some(Predicate::Plugin(value)),
        "name" => (!value.is_empty()).then_some(Predicate::Name(value)),
        "desc" | "description" => (!value.is_empty()).then_some(Predicate::Desc(value)),
        "alias" => (!value.is_empty()).then_some(Predicate::Alias(value)),
        "tag" => (!value.is_empty()).then_some(Predicate::Tag(value)),
        "has" => (!value.is_empty()).then_some(Predicate::Has(value)),
        _ => None,
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let chars = input.chars().peekable();
    let mut in_quote: Option<char> = None;

    for c in chars {
        match in_quote {
            Some(q) if c == q => {
                in_quote = None;
                current.push(c);
            }
            Some(_) => current.push(c),
            None if c == '"' || c == '\'' => {
                in_quote = Some(c);
                current.push(c);
            }
            None if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
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

fn quoted(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_type_and_free_text() {
        let parsed = parse_advanced_query(":type:script git");
        assert_eq!(parsed.free_text, "git");
        assert_eq!(
            parsed.predicates,
            vec![Predicate::Type(ArtifactKind::Script)]
        );
        assert!(parsed.source_filters.is_empty());
    }

    #[test]
    fn source_filter_tokens_parse_anywhere() {
        let parsed = parse_filter_query("open project files:").unwrap();
        assert_eq!(parsed.free_text, "open project");
        assert!(parsed.source_filters.allows(RootUnifiedSourceFilter::Files));
        assert!(!parsed.source_filters.allows(RootUnifiedSourceFilter::Notes));
    }

    #[test]
    fn source_filter_prefix_form_strips_token() {
        let parsed = parse_filter_query("n: meeting notes").unwrap();
        assert_eq!(parsed.free_text, "meeting notes");
        assert!(parsed.source_filters.allows(RootUnifiedSourceFilter::Notes));
    }

    #[test]
    fn multiple_source_filters_are_additive() {
        let parsed = parse_filter_query("n: invoice c:").unwrap();
        assert_eq!(parsed.free_text, "invoice");
        assert!(parsed.source_filters.allows(RootUnifiedSourceFilter::Notes));
        assert!(parsed
            .source_filters
            .allows(RootUnifiedSourceFilter::ClipboardHistory));
        assert!(!parsed.source_filters.allows(RootUnifiedSourceFilter::Files));
    }

    #[test]
    fn unknown_colon_token_stays_literal() {
        assert!(parse_filter_query("deploy x:").is_none());
    }

    #[test]
    fn quoted_source_filter_is_literal() {
        assert!(parse_filter_query("search \"f:\"").is_none());
    }

    #[test]
    fn legacy_colon_prefix_source_filter_is_not_committed_syntax() {
        assert!(parse_filter_query(":f png").is_none());
    }

    #[test]
    fn source_filter_exclusion_wins() {
        let parsed = parse_filter_query("files: -files: png").unwrap();
        assert_eq!(parsed.free_text, "png");
        assert!(parsed
            .source_filters
            .includes(RootUnifiedSourceFilter::Files));
        assert!(parsed
            .source_filters
            .excludes(RootUnifiedSourceFilter::Files));
        assert!(!parsed.source_filters.allows(RootUnifiedSourceFilter::Files));
    }

    #[test]
    fn parses_shortcut_true() {
        let parsed = parse_advanced_query(":shortcut:true foo");
        assert_eq!(
            parsed.predicates,
            vec![Predicate::HasShortcut(ShortcutPredicate::Any)]
        );
        assert_eq!(parsed.free_text, "foo");
    }

    #[test]
    fn parses_shortcut_any_alias() {
        let parsed = parse_advanced_query(":shortcut:any deploy");
        assert_eq!(
            parsed.predicates,
            vec![Predicate::HasShortcut(ShortcutPredicate::Any)]
        );
        assert_eq!(parsed.free_text, "deploy");
    }

    #[test]
    fn parses_shortcut_literal() {
        let parsed = parse_advanced_query(":shortcut:cmd+g");
        assert_eq!(
            parsed.predicates,
            vec![Predicate::HasShortcut(ShortcutPredicate::Literal(
                "cmd+g".to_string()
            ))]
        );
    }

    #[test]
    fn parses_negated_type() {
        let parsed = parse_advanced_query(":-type:script deploy");
        assert_eq!(
            parsed.predicates,
            vec![Predicate::Negate(Box::new(Predicate::Type(
                ArtifactKind::Script
            )))]
        );
        assert_eq!(parsed.free_text, "deploy");
    }

    #[test]
    fn parses_meta_path() {
        let parsed = parse_advanced_query(":meta.domain:calendar has:schema event");
        assert_eq!(
            parsed.predicates,
            vec![
                Predicate::MetaPath {
                    path: "domain".to_string(),
                    value: "calendar".to_string()
                },
                Predicate::Has("schema".to_string())
            ]
        );
        assert_eq!(parsed.free_text, "event");
    }

    #[test]
    fn kind_is_alias_for_type() {
        let parsed = parse_advanced_query(":kind:skill triage");
        assert_eq!(
            parsed.predicates,
            vec![Predicate::Type(ArtifactKind::Skill)]
        );
    }

    #[test]
    fn type_agent_and_issue_parse_to_dedicated_kinds() {
        let agent = parse_advanced_query(":type:agent foo");
        assert_eq!(agent.predicates, vec![Predicate::Type(ArtifactKind::Agent)]);
        let issue = parse_advanced_query(":type:issue bar");
        assert_eq!(issue.predicates, vec![Predicate::Type(ArtifactKind::Issue)]);
        let issue_alt = parse_advanced_query(":type:scriptissue bar");
        assert_eq!(
            issue_alt.predicates,
            vec![Predicate::Type(ArtifactKind::Issue)]
        );
    }

    #[test]
    fn unknown_qualifier_falls_through_to_free_text() {
        let parsed = parse_advanced_query(":wat:foo bar baz");
        assert_eq!(parsed.predicates.len(), 0);
        assert_eq!(parsed.free_text, "wat:foo bar baz");
    }

    #[test]
    fn parses_hash_tag_sugar_and_canonical_tag_qualifier() {
        let parsed = parse_advanced_query(":#work type:script deploy");
        assert_eq!(
            parsed.predicates,
            vec![
                Predicate::Tag("work".to_string()),
                Predicate::Type(ArtifactKind::Script)
            ]
        );
        assert_eq!(parsed.free_text, "deploy");

        let canonical = parse_advanced_query(":tag:client/acme -#archived");
        assert_eq!(
            canonical.predicates,
            vec![
                Predicate::Tag("client/acme".to_string()),
                Predicate::Negate(Box::new(Predicate::Tag("archived".to_string())))
            ]
        );
    }

    #[test]
    fn quoted_values_preserve_spaces() {
        let parsed = parse_advanced_query(":source:\"github issues\" triage");
        assert_eq!(
            parsed.predicates,
            vec![Predicate::Source("github issues".to_string())]
        );
        assert_eq!(parsed.free_text, "triage");
    }

    #[test]
    fn bare_colon_returns_empty_query() {
        let parsed = parse_advanced_query(":");
        assert_eq!(parsed.free_text, "");
        assert!(parsed.predicates.is_empty());
    }
}
