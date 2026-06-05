use super::capture::{is_capture_target_registered, parse_capture_with_targets, CaptureParse};
use super::payload::picker_visible_capture_targets;
use super::payload::{
    AdvancedQuery, ArgvInvocation, CaptureInvocation, IncompleteKind, IncompleteSyntax,
};
use super::query::{parse_advanced_query, parse_filter_query};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuSyntaxParse {
    None,
    AdvancedQuery(AdvancedQuery),
    Capture(CaptureInvocation),
    Argv(ArgvInvocation),
    Incomplete(IncompleteSyntax),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuSyntaxParseConfig {
    pub argv_enabled: bool,
}

impl MenuSyntaxParseConfig {
    pub const fn default() -> Self {
        Self { argv_enabled: true }
    }
}

pub fn parse(input: &str) -> MenuSyntaxParse {
    parse_with_capture_targets(input, &[])
}

pub fn parse_with_capture_targets(
    input: &str,
    registered_capture_targets: &[String],
) -> MenuSyntaxParse {
    parse_with_config_and_capture_targets(
        input,
        MenuSyntaxParseConfig {
            argv_enabled: argv_enabled(),
        },
        registered_capture_targets,
    )
}

pub fn parse_with_config(input: &str, config: MenuSyntaxParseConfig) -> MenuSyntaxParse {
    parse_with_config_and_capture_targets(input, config, &[])
}

pub fn parse_with_config_and_capture_targets(
    input: &str,
    config: MenuSyntaxParseConfig,
    registered_capture_targets: &[String],
) -> MenuSyntaxParse {
    if input.is_empty() {
        return MenuSyntaxParse::None;
    }

    if let Some(first) = input.chars().next() {
        if first == ':' {
            let query = parse_advanced_query(input);
            let rest = input.strip_prefix(':').unwrap_or(input).trim();
            if !rest.is_empty()
                && (!query.predicates.is_empty() || !query.source_filters.is_empty())
            {
                return MenuSyntaxParse::AdvancedQuery(query);
            }
            return MenuSyntaxParse::Incomplete(IncompleteSyntax {
                kind: IncompleteKind::BareQueryPrefix,
                hint: "Choose a filter: files, notes, clipboard, type, tag, shortcut".to_string(),
            });
        }

        if first == ';' || first == '+' {
            let rest = input
                .strip_prefix(';')
                .or_else(|| input.strip_prefix('+'))
                .unwrap_or(input);
            if rest.is_empty() {
                return MenuSyntaxParse::Incomplete(IncompleteSyntax {
                    kind: IncompleteKind::BareCapturePrefix,
                    hint: format!(
                        "Choose a capture target: {}",
                        picker_visible_capture_targets().join(", ")
                    ),
                });
            }
            let head_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let head = &rest[..head_end];
            if !is_capture_target_registered(head, registered_capture_targets) {
                return MenuSyntaxParse::None;
            }
            return finalize_capture(input, registered_capture_targets);
        }

        if first == '>' {
            if !config.argv_enabled {
                return MenuSyntaxParse::None;
            }
            return parse_argv(input);
        }
    }

    if let Some(colon_idx) = input.find(':') {
        let head = &input[..colon_idx];
        if super::payload::source_for_head(&input[..=colon_idx]).is_some() {
            if let Some(query) = parse_filter_query(input) {
                return MenuSyntaxParse::AdvancedQuery(query);
            }
        }
        if !head.is_empty()
            && !head.contains(char::is_whitespace)
            && is_capture_target_registered(head, registered_capture_targets)
        {
            return finalize_capture(input, registered_capture_targets);
        }
    }

    if let Some(query) = parse_filter_query(input) {
        return MenuSyntaxParse::AdvancedQuery(query);
    }

    MenuSyntaxParse::None
}

fn finalize_capture(input: &str, registered_capture_targets: &[String]) -> MenuSyntaxParse {
    match parse_capture_with_targets(input, registered_capture_targets) {
        CaptureParse::Ok(inv) => {
            if capture_has_content(&inv) {
                MenuSyntaxParse::Capture(inv)
            } else {
                MenuSyntaxParse::Incomplete(IncompleteSyntax {
                    kind: IncompleteKind::MissingCaptureBody(inv.target.clone()),
                    hint: missing_body_hint(&inv.target),
                })
            }
        }
        CaptureParse::Incomplete(s) => MenuSyntaxParse::Incomplete(s),
    }
}

fn capture_has_content(inv: &CaptureInvocation) -> bool {
    if !inv.body.trim().is_empty() {
        return true;
    }
    if inv.target == "link" && inv.url.is_some() {
        return true;
    }
    super::capture_schema::builtin_schema(&inv.target)
        .map(|schema| schema.missing_required(inv).is_empty())
        .unwrap_or(false)
}

fn missing_body_hint(target: &str) -> String {
    match target {
        "link" => {
            "Provide a URL or body for ;link (e.g. `;link https://zed.dev #rust`)".to_string()
        }
        other => format!("Type what you want to capture for ;{other} (body, tags, dates, etc.)"),
    }
}

fn parse_argv(input: &str) -> MenuSyntaxParse {
    let rest = input.strip_prefix('>').unwrap_or(input);
    if rest.trim().is_empty() {
        return MenuSyntaxParse::Incomplete(IncompleteSyntax {
            kind: IncompleteKind::BareArgvPrefix,
            hint: "Type a command name: ><name> [-- argv...]".to_string(),
        });
    }

    let (head_part, argv_part) = if let Some(idx) = rest.find(" -- ") {
        (&rest[..idx], Some(&rest[idx + 4..]))
    } else {
        (rest, None)
    };

    let mut tokens = split_argv(head_part);
    if tokens.is_empty() {
        return MenuSyntaxParse::Incomplete(IncompleteSyntax {
            kind: IncompleteKind::BareArgvPrefix,
            hint: "Type a command name: ><name> [-- argv...]".to_string(),
        });
    }

    let head = tokens.remove(0);
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    let mut argv: Vec<String> = Vec::new();

    for token in tokens {
        if let Some(tag) = token.strip_prefix('#') {
            if !tag.is_empty() {
                tags.push(tag.to_string());
            }
            continue;
        }
        if let Some((key, value)) = token.split_once(':') {
            if !key.is_empty() && !value.is_empty() {
                fields.push((key.to_string(), unquote(value)));
                continue;
            }
        }
        argv.push(token);
    }

    if let Some(argv_str) = argv_part {
        argv.extend(split_argv(argv_str));
    };

    MenuSyntaxParse::Argv(ArgvInvocation {
        head,
        fields,
        tags,
        argv,
        raw: input.to_string(),
    })
}

fn split_argv(argv_str: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let chars = argv_str.chars().peekable();
    let mut in_quote: Option<char> = None;

    for c in chars {
        match in_quote {
            Some(q) if c == q => {
                in_quote = None;
            }
            Some(_) => current.push(c),
            None if c == '"' || c == '\'' => in_quote = Some(c),
            None if c.is_whitespace() => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            None => current.push(c),
        }
    }
    if !current.is_empty() {
        out.push(current);
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

pub fn argv_enabled() -> bool {
    std::env::var("KIT_MENU_SYNTAX_ARGV")
        .map(|v| v != "0")
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::payload::{
        ArtifactKind, CaptureAlias, Predicate, RootUnifiedSourceFilter,
    };

    const ARGV_OFF: MenuSyntaxParseConfig = MenuSyntaxParseConfig {
        argv_enabled: false,
    };
    const ARGV_ON: MenuSyntaxParseConfig = MenuSyntaxParseConfig { argv_enabled: true };

    #[test]
    fn empty_input_returns_none() {
        assert_eq!(parse(""), MenuSyntaxParse::None);
    }

    #[test]
    fn plain_text_returns_none() {
        assert_eq!(parse("git deploy"), MenuSyntaxParse::None);
        assert_eq!(parse("some search"), MenuSyntaxParse::None);
        assert_eq!(parse("#work"), MenuSyntaxParse::None);
        assert_eq!(parse("hello world!"), MenuSyntaxParse::None);
    }

    #[test]
    fn existing_triggers_return_none() {
        for trigger in ["~", "~/Desktop", "/", "@", ">", "?"] {
            let result = parse(trigger);
            assert_eq!(
                result,
                MenuSyntaxParse::None,
                "existing trigger {trigger} must not be claimed by menu_syntax::parse"
            );
        }
    }

    #[test]
    fn colon_prefix_routes_to_advanced_query() {
        let result = parse("type:script git");
        match result {
            MenuSyntaxParse::AdvancedQuery(q) => {
                assert_eq!(q.predicates, vec![Predicate::Type(ArtifactKind::Script)]);
                assert_eq!(q.free_text, "git");
                assert!(q.source_filters.is_empty());
            }
            other => panic!("expected AdvancedQuery, got {other:?}"),
        }
    }

    #[test]
    fn inline_source_filter_routes_to_query_mode() {
        match parse("meeting n:") {
            MenuSyntaxParse::AdvancedQuery(q) => {
                assert_eq!(q.free_text, "meeting");
                assert!(q.predicates.is_empty());
                assert!(q.source_filters.allows(RootUnifiedSourceFilter::Notes));
            }
            other => panic!("expected source-filter query, got {other:?}"),
        }
    }

    #[test]
    fn source_filter_prefix_routes_to_query_mode() {
        match parse("f: meeting") {
            MenuSyntaxParse::AdvancedQuery(q) => {
                assert_eq!(q.free_text, "meeting");
                assert!(q.predicates.is_empty());
                assert!(q.source_filters.allows(RootUnifiedSourceFilter::Files));
            }
            other => panic!("expected source-filter query, got {other:?}"),
        }
    }

    #[test]
    fn capture_keyword_still_owns_input_before_source_filters() {
        match parse("note: meeting f:") {
            MenuSyntaxParse::Capture(_) => {}
            other => panic!("expected Capture, got {other:?}"),
        }
    }

    #[test]
    fn bare_colon_is_incomplete() {
        match parse(":") {
            MenuSyntaxParse::Incomplete(s) => {
                assert!(matches!(s.kind, IncompleteKind::BareQueryPrefix))
            }
            other => panic!("expected Incomplete, got {other:?}"),
        }
    }

    #[test]
    fn leading_colon_complete_type_filter_parses_advanced_query() {
        match parse(":type:skill review") {
            MenuSyntaxParse::AdvancedQuery(q) => {
                assert_eq!(q.free_text, "review");
                assert_eq!(q.predicates, vec![Predicate::Type(ArtifactKind::Skill)]);
            }
            other => panic!("expected AdvancedQuery, got {other:?}"),
        }
    }

    #[test]
    fn leading_colon_partial_filter_stays_incomplete() {
        for raw in [":", ":typ", ":type:", ":type:s", ":has:sh", ":#"] {
            match parse(raw) {
                MenuSyntaxParse::Incomplete(s) => {
                    assert!(
                        matches!(s.kind, IncompleteKind::BareQueryPrefix),
                        "expected BareQueryPrefix for {raw:?}, got {s:?}"
                    );
                }
                other => panic!("expected BareQueryPrefix for {raw:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn semicolon_routes_to_capture() {
        match parse(";todo Renew passport #errands p1") {
            MenuSyntaxParse::Capture(inv) => {
                assert_eq!(inv.target, "todo");
                assert_eq!(inv.alias_form, CaptureAlias::CapturePrefix);
                assert_eq!(inv.body, "Renew passport");
                assert_eq!(inv.tags, vec!["errands".to_string()]);
                assert_eq!(inv.priority, Some(1));
            }
            other => panic!("expected Capture, got {other:?}"),
        }
    }

    #[test]
    fn keyword_alias_routes_to_capture() {
        match parse("note: Decision to ship parser first #menu-syntax") {
            MenuSyntaxParse::Capture(inv) => {
                assert_eq!(inv.target, "note");
                assert_eq!(inv.alias_form, CaptureAlias::Keyword);
                assert_eq!(inv.tags, vec!["menu-syntax".to_string()]);
            }
            other => panic!("expected Capture, got {other:?}"),
        }
    }

    #[test]
    fn unknown_keyword_head_does_not_route_to_capture() {
        assert_eq!(parse("localhost:3000 check"), MenuSyntaxParse::None);
        assert_eq!(parse("not-a-target: stuff"), MenuSyntaxParse::None);
    }

    #[test]
    fn capture_prefix_bare_is_incomplete() {
        for input in [";", "+"] {
            match parse(input) {
                MenuSyntaxParse::Incomplete(s) => {
                    assert!(matches!(s.kind, IncompleteKind::BareCapturePrefix));
                }
                other => panic!("expected Incomplete for {input}, got {other:?}"),
            }
        }
    }

    #[test]
    fn unknown_capture_prefix_head_falls_back_to_normal_search() {
        assert_eq!(parse(";github"), MenuSyntaxParse::None);
        assert_eq!(parse(";github issue"), MenuSyntaxParse::None);
        assert_eq!(parse("+github"), MenuSyntaxParse::None);
        assert_eq!(parse("+1"), MenuSyntaxParse::None);
        assert_eq!(parse("+react component"), MenuSyntaxParse::None);
    }

    #[test]
    fn prefix_ownership_conformance_matrix() {
        let registered = vec!["github".to_string()];
        let cases: &[(&str, &[String], &str)] = &[
            (";", &[], "bare-capture-prefix"),
            (";todo", &[], "missing-body:todo"),
            (";todo body", &[], "capture:todo:prefix"),
            (";unknown", &[], "none"),
            (";unknown body", &[], "none"),
            ("+", &[], "bare-capture-prefix"),
            ("+todo body", &[], "capture:todo:prefix"),
            ("+unknown", &[], "none"),
            ("+unknown body", &[], "none"),
            ("todo: body", &[], "advanced-query"),
            ("unknown: body", &[], "none"),
            ("localhost:3000", &[], "none"),
            ("#tag", &[], "none"),
            (";github body", &registered, "capture:github:prefix"),
            ("+github body", &registered, "capture:github:prefix"),
            ("github: body", &registered, "capture:github:keyword"),
        ];

        for (input, targets, expected) in cases {
            let actual = parse_shape(parse_with_capture_targets(input, targets));
            assert_eq!(
                actual, *expected,
                "parse boundary mismatch for input {input:?}"
            );
        }
    }

    #[test]
    fn registered_capture_target_claims_legacy_plus_and_keyword_forms() {
        let targets = vec!["github".to_string()];
        match parse_with_capture_targets("+github issue #bug", &targets) {
            MenuSyntaxParse::Capture(inv) => {
                assert_eq!(inv.target, "github");
                assert_eq!(inv.body, "issue");
                assert_eq!(inv.tags, vec!["bug".to_string()]);
            }
            other => panic!("expected registered +github capture, got {other:?}"),
        }
        match parse_with_capture_targets("github: issue", &targets) {
            MenuSyntaxParse::Capture(inv) => {
                assert_eq!(inv.target, "github");
                assert_eq!(inv.alias_form, CaptureAlias::Keyword);
            }
            other => panic!("expected registered github: capture, got {other:?}"),
        }
    }

    #[test]
    fn demo_dynamic_capture_targets_parse_only_when_registered() {
        assert_eq!(parse("+github Local issue #bug"), MenuSyntaxParse::None);

        let targets = vec![
            "github".to_string(),
            "expense".to_string(),
            "snippet".to_string(),
            "fixture".to_string(),
        ];
        match parse_with_capture_targets("+github Local issue repo=script-kit/gpui #bug", &targets)
        {
            MenuSyntaxParse::Capture(inv) => {
                assert_eq!(inv.target, "github");
                assert_eq!(inv.body, "Local issue");
                assert_eq!(
                    inv.kv,
                    vec![("repo".to_string(), "script-kit/gpui".to_string())]
                );
                assert_eq!(inv.tags, vec!["bug".to_string()]);
            }
            other => panic!("expected registered +github capture, got {other:?}"),
        }

        assert_eq!(
            parse_with_capture_targets("+jira Ticket #bug", &targets),
            MenuSyntaxParse::None
        );
    }

    #[test]
    fn known_capture_target_without_content_is_incomplete() {
        for input in [";todo", ";note", ";cal", ";social", "note:"] {
            match parse(input) {
                MenuSyntaxParse::Incomplete(s) => {
                    assert!(
                        matches!(s.kind, IncompleteKind::MissingCaptureBody(_)),
                        "expected MissingCaptureBody for '{input}', got {s:?}"
                    );
                }
                other => panic!("expected Incomplete for '{input}', got {other:?}"),
            }
        }

        match parse("todo:") {
            MenuSyntaxParse::AdvancedQuery(query) => {
                assert_eq!(query.free_text, "");
                assert!(query.source_filters.allows(RootUnifiedSourceFilter::Todo));
            }
            other => panic!("expected todo source filter for 'todo:', got {other:?}"),
        }
    }

    #[test]
    fn selected_note_ref_with_metadata_is_complete_capture() {
        match parse(";note @note:550e8400-e29b-41d4-a716-446655440000 due:tomorrow") {
            MenuSyntaxParse::Capture(inv) => {
                assert_eq!(inv.target, "note");
                assert!(inv.body.trim().is_empty());
                assert_eq!(inv.date_phrases.len(), 1);
            }
            other => panic!("expected selected note metadata update capture, got {other:?}"),
        }
    }

    #[test]
    fn selected_note_ref_without_metadata_still_needs_content() {
        match parse(";note @note:550e8400-e29b-41d4-a716-446655440000") {
            MenuSyntaxParse::Incomplete(s) => {
                assert!(matches!(s.kind, IncompleteKind::MissingCaptureBody(_)));
            }
            other => {
                panic!("expected selected note without metadata to be incomplete, got {other:?}")
            }
        }
    }

    #[test]
    fn link_capture_allows_url_without_body() {
        assert!(matches!(
            parse(";link https://zed.dev"),
            MenuSyntaxParse::Capture(_)
        ));
        assert!(matches!(
            parse("link: url:https://example.com"),
            MenuSyntaxParse::Capture(_)
        ));
    }

    #[test]
    fn link_capture_without_url_or_body_is_incomplete() {
        match parse(";link") {
            MenuSyntaxParse::Incomplete(s) => {
                assert!(matches!(s.kind, IncompleteKind::MissingCaptureBody(_)));
            }
            other => panic!("expected Incomplete, got {other:?}"),
        }
    }

    #[test]
    fn bang_prefix_can_still_be_disabled_by_config() {
        assert_eq!(
            parse_with_config(">deploy -- prod", ARGV_OFF),
            MenuSyntaxParse::None
        );
    }

    #[test]
    fn bang_prefix_parses_when_flag_enabled() {
        match parse_with_config(">deploy -- prod --dry-run", ARGV_ON) {
            MenuSyntaxParse::Argv(a) => {
                assert_eq!(a.head, "deploy");
                assert!(a.fields.is_empty());
                assert!(a.tags.is_empty());
                assert_eq!(a.argv, vec!["prod".to_string(), "--dry-run".to_string()]);
            }
            other => panic!("expected Argv, got {other:?}"),
        }
    }

    #[test]
    fn bang_prefix_parses_fields_tags_and_raw_argv() {
        match parse(">deploy env:prod #release -- --dry-run") {
            MenuSyntaxParse::Argv(a) => {
                assert_eq!(a.head, "deploy");
                assert_eq!(a.fields, vec![("env".to_string(), "prod".to_string())]);
                assert_eq!(a.tags, vec!["release".to_string()]);
                assert_eq!(a.argv, vec!["--dry-run".to_string()]);
            }
            other => panic!("expected Argv, got {other:?}"),
        }
    }

    fn parse_shape(parsed: MenuSyntaxParse) -> String {
        match parsed {
            MenuSyntaxParse::None => "none".to_string(),
            MenuSyntaxParse::AdvancedQuery(_) => "advanced-query".to_string(),
            MenuSyntaxParse::Argv(_) => "argv".to_string(),
            MenuSyntaxParse::Capture(inv) => {
                let alias = match inv.alias_form {
                    CaptureAlias::CapturePrefix => "prefix",
                    CaptureAlias::Keyword => "keyword",
                };
                format!("capture:{}:{alias}", inv.target)
            }
            MenuSyntaxParse::Incomplete(s) => match s.kind {
                IncompleteKind::BareCapturePrefix => "bare-capture-prefix".to_string(),
                IncompleteKind::BareQueryPrefix => "bare-query-prefix".to_string(),
                IncompleteKind::BareArgvPrefix => "bare-argv-prefix".to_string(),
                IncompleteKind::MissingCaptureBody(target) => format!("missing-body:{target}"),
                IncompleteKind::UnknownCaptureTarget(target) => {
                    format!("unknown-capture-target:{target}")
                }
            },
        }
    }
}
