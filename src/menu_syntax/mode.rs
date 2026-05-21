use std::ops::Range;

use super::capture::is_capture_target_registered;
use super::fragments::MenuSyntaxFragmentRole;
use super::parse::{parse, parse_with_capture_targets, MenuSyntaxParse};
use super::payload::{
    AdvancedQuery, ArgvInvocation, CaptureInvocation, IncompleteKind, IncompleteSyntax,
};

/// Raw-guarded mode state for ScriptList power syntax.
///
/// Parse at input-change boundaries, not inside result grouping. The raw guard
/// prevents a render frame from applying a parse computed for newer input
/// against a stale `computed_filter_text` (the launcher coalesces filter text
/// with an 8ms debounce, so the two can disagree for a tick).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSyntaxMode {
    raw: String,
    parse: MenuSyntaxParse,
    capture_targets: Vec<String>,
}

impl Default for MenuSyntaxMode {
    fn default() -> Self {
        Self {
            raw: String::new(),
            parse: MenuSyntaxParse::None,
            capture_targets: Vec::new(),
        }
    }
}

impl MenuSyntaxMode {
    pub fn from_input(raw: &str) -> Self {
        Self::from_input_with_capture_targets(raw, &[])
    }

    pub fn from_input_with_capture_targets(raw: &str, capture_targets: &[String]) -> Self {
        Self {
            raw: raw.to_string(),
            parse: if capture_targets.is_empty() {
                parse(raw)
            } else {
                parse_with_capture_targets(raw, capture_targets)
            },
            capture_targets: capture_targets.to_vec(),
        }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn parse(&self) -> &MenuSyntaxParse {
        &self.parse
    }

    pub fn parse_for(&self, raw: &str) -> Option<&MenuSyntaxParse> {
        (self.raw == raw).then_some(&self.parse)
    }

    pub fn advanced_query_for(&self, raw: &str) -> Option<&AdvancedQuery> {
        match self.parse_for(raw)? {
            MenuSyntaxParse::AdvancedQuery(query) => Some(query),
            _ => None,
        }
    }

    pub fn capture_for(&self, raw: &str) -> Option<&CaptureInvocation> {
        match self.parse_for(raw)? {
            MenuSyntaxParse::Capture(invocation) => Some(invocation),
            _ => None,
        }
    }

    pub fn command_for(&self, raw: &str) -> Option<&ArgvInvocation> {
        match self.parse_for(raw)? {
            MenuSyntaxParse::Argv(invocation) => Some(invocation),
            _ => None,
        }
    }

    pub fn incomplete_for(&self, raw: &str) -> Option<&IncompleteSyntax> {
        match self.parse_for(raw)? {
            MenuSyntaxParse::Incomplete(incomplete) => Some(incomplete),
            _ => None,
        }
    }

    pub fn incomplete_hint_for(&self, raw: &str) -> Option<&str> {
        self.incomplete_for(raw).map(|s| s.hint.as_str())
    }

    pub fn is_menu_syntax_for(&self, raw: &str) -> bool {
        !matches!(self.parse_for(raw), None | Some(MenuSyntaxParse::None))
    }

    /// Returns true once a known capture target has been committed and the
    /// input is now a text composer instead of launcher search.
    pub fn capture_composer_owns_input_for(&self, raw: &str) -> bool {
        let Some(parse) = self.parse_for(raw) else {
            return false;
        };
        match parse {
            MenuSyntaxParse::Capture(_) => true,
            MenuSyntaxParse::Incomplete(incomplete) => match &incomplete.kind {
                IncompleteKind::MissingCaptureBody(target) => {
                    capture_target_is_committed_with_targets(raw, target, &self.capture_targets)
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns true once `!` has opted into command invocation. Command text
    /// should never fall through to fuzzy search or shell execution.
    pub fn command_owns_input_for(&self, raw: &str) -> bool {
        matches!(
            self.parse_for(raw),
            Some(MenuSyntaxParse::Argv(_))
                | Some(MenuSyntaxParse::Incomplete(IncompleteSyntax {
                    kind: IncompleteKind::BareArgvPrefix,
                    ..
                }))
        )
    }
}

/// The text grouping/search should actually fuzzy-match against.
///
/// For advanced-query mode this is the free-text portion (`:type:script git`
/// searches for `git`). For every other mode the raw input is the search text.
pub fn free_text_for_search<'a>(mode: &'a MenuSyntaxMode, raw: &'a str) -> &'a str {
    match mode.advanced_query_for(raw) {
        Some(query) => query.free_text.as_str(),
        None => raw,
    }
}

pub fn capture_body_boundary_has_started(raw: &str) -> bool {
    capture_body_boundary_has_started_with_targets(raw, &[])
}

pub fn capture_body_boundary_has_started_with_targets(
    raw: &str,
    registered_targets: &[String],
) -> bool {
    let raw = raw.trim_start();
    if let Some(rest) = raw.strip_prefix(';') {
        let target_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        if target_end == 0 {
            return false;
        }
        let target = &rest[..target_end];
        return is_capture_target_registered(target, registered_targets) && target_end < rest.len();
    }

    let Some(colon_idx) = raw.find(':') else {
        return false;
    };
    let target = &raw[..colon_idx];
    if target.is_empty()
        || target.contains(char::is_whitespace)
        || !is_capture_target_registered(target, registered_targets)
    {
        return false;
    }
    true
}

fn capture_target_is_committed_with_targets(
    raw: &str,
    target: &str,
    registered_targets: &[String],
) -> bool {
    let raw = raw.trim_start();
    if let Some(rest) = raw.strip_prefix(';').or_else(|| raw.strip_prefix('+')) {
        let target_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        if target_end == 0 {
            return false;
        }
        let raw_target = &rest[..target_end];
        return raw_target.eq_ignore_ascii_case(target)
            && is_capture_target_registered(raw_target, registered_targets);
    }

    let Some(colon_idx) = raw.find(':') else {
        return false;
    };
    let raw_target = &raw[..colon_idx];
    !raw_target.is_empty()
        && !raw_target.contains(char::is_whitespace)
        && raw_target.eq_ignore_ascii_case(target)
        && is_capture_target_registered(raw_target, registered_targets)
}

/// Byte span of the "prefix chrome" that should get accent styling in the input
/// field, for discoverability. `None` means no highlight.
///
/// - `type:script git` → `0..11` (`type:script`)
/// - `+todo Renew passport` → `0..5` (`+todo`)
/// - `note: Renew passport` → `0..5` (`note:`)
/// - `+` → `0..1`
/// - plain text → `None`
pub fn prefix_span_for_input(raw: &str) -> Option<Range<usize>> {
    prefix_span_for_input_with_targets(raw, &[])
}

pub fn prefix_span_for_input_with_targets(
    raw: &str,
    registered_targets: &[String],
) -> Option<Range<usize>> {
    if raw.is_empty() {
        return None;
    }
    if let Some(source_head) = source_filter_head_span(raw).into_iter().next() {
        if source_head.range.start == 0 {
            return Some(source_head.range);
        }
    }
    let bytes = raw.as_bytes();
    if bytes[0] == b':' {
        let head_end = raw.find(char::is_whitespace).unwrap_or(raw.len());
        return Some(0..head_end);
    }
    if bytes[0] == b';' {
        let rest = &raw[1..];
        if rest.is_empty() {
            return Some(0..1);
        }
        let head_end_in_rest = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let head = &rest[..head_end_in_rest];
        if is_capture_target_registered(head, registered_targets)
            || super::payload::KNOWN_CAPTURE_TARGETS
                .iter()
                .any(|target| target.starts_with(head))
            || registered_targets
                .iter()
                .any(|target| target.starts_with(head))
        {
            return Some(0..1 + head_end_in_rest);
        }
        return None;
    }
    if bytes[0] == b'>' {
        let rest = &raw[1..];
        if rest.is_empty() {
            return Some(0..1);
        }
        let head_end_in_rest = rest.find(char::is_whitespace).unwrap_or(rest.len());
        if head_end_in_rest > 0 {
            return Some(0..1 + head_end_in_rest);
        }
        return Some(0..1);
    }
    if let Some(colon_idx) = raw.find(':') {
        let head = &raw[..colon_idx];
        if !head.is_empty()
            && !head.contains(char::is_whitespace)
            && is_capture_target_registered(head, registered_targets)
        {
            return Some(0..colon_idx + 1);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSyntaxInputSpan {
    pub range: Range<usize>,
    pub role: MenuSyntaxFragmentRole,
}

pub fn input_spans_for_input(raw: &str) -> Vec<MenuSyntaxInputSpan> {
    input_spans_for_input_with_targets(raw, &[])
}

pub fn input_spans_for_input_with_targets(
    raw: &str,
    registered_targets: &[String],
) -> Vec<MenuSyntaxInputSpan> {
    let mut spans = Vec::new();
    let prefix = prefix_span_for_input_with_targets(raw, registered_targets);
    if let Some(range) = prefix.clone() {
        spans.push(MenuSyntaxInputSpan {
            range,
            role: MenuSyntaxFragmentRole::Prefix,
        });
    }
    spans.extend(source_filter_head_span(raw));

    let invocation = match parse_with_capture_targets(raw, registered_targets) {
        MenuSyntaxParse::Capture(invocation) => Some(invocation),
        _ => None,
    };

    if let Some(invocation) = invocation.as_ref() {
        spans.extend(capture_token_spans(raw));
        if capture_accepts_nl_fragments(invocation) {
            let clock = crate::menu_syntax::date::MenuSyntaxClock::local_now();
            let resolved =
                crate::menu_syntax::nl_phrase::resolve_capture_nl_phrase(invocation, &clock);
            let search_start = capture_body_start(raw).unwrap_or(0);
            for fragment in resolved.fragments {
                if matches!(fragment.role, MenuSyntaxFragmentRole::Subject) {
                    continue;
                }
                if let Some(range) = find_fragment_range(raw, search_start, &fragment.source) {
                    spans.push(MenuSyntaxInputSpan {
                        range,
                        role: fragment.role,
                    });
                }
            }
        }
    }

    normalize_input_spans(raw, spans, prefix)
}

pub fn input_span_role_name(role: MenuSyntaxFragmentRole) -> &'static str {
    match role {
        MenuSyntaxFragmentRole::Prefix => "prefix",
        MenuSyntaxFragmentRole::Subject => "subject",
        MenuSyntaxFragmentRole::Date => "date",
        MenuSyntaxFragmentRole::DateRange => "dateRange",
        MenuSyntaxFragmentRole::Duration => "duration",
        MenuSyntaxFragmentRole::Recurrence => "recurrence",
        MenuSyntaxFragmentRole::Kv => "kv",
        MenuSyntaxFragmentRole::Tag => "tag",
        MenuSyntaxFragmentRole::Url => "url",
        MenuSyntaxFragmentRole::Priority => "priority",
        MenuSyntaxFragmentRole::ObjectRef => "objectRef",
        MenuSyntaxFragmentRole::Unresolved => "unresolved",
    }
}

fn capture_accepts_nl_fragments(invocation: &CaptureInvocation) -> bool {
    if invocation.target.eq_ignore_ascii_case("cal")
        || invocation.target.eq_ignore_ascii_case("mcal")
    {
        return true;
    }
    let Some(schema) = crate::menu_syntax::builtin_schema(&invocation.target) else {
        return false;
    };
    schema
        .required
        .iter()
        .chain(schema.optional.iter())
        .any(|requirement| {
            matches!(
                requirement,
                super::capture_schema::FieldRequirement::AnyDate
                    | super::capture_schema::FieldRequirement::DateRole(_)
            )
        })
}

fn capture_body_start(raw: &str) -> Option<usize> {
    if let Some(rest) = raw.strip_prefix(';').or_else(|| raw.strip_prefix('+')) {
        let head_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let start = 1 + head_end;
        return Some(
            start
                + raw[start..]
                    .bytes()
                    .take_while(|b| b.is_ascii_whitespace())
                    .count(),
        );
    }
    let colon_idx = raw.find(':')?;
    let start = colon_idx + 1;
    Some(
        start
            + raw[start..]
                .bytes()
                .take_while(|b| b.is_ascii_whitespace())
                .count(),
    )
}

#[derive(Debug, Clone)]
struct RawToken<'a> {
    text: &'a str,
    range: Range<usize>,
}

fn capture_token_spans(raw: &str) -> Vec<MenuSyntaxInputSpan> {
    raw_tokens(raw, capture_body_start(raw).unwrap_or(0))
        .into_iter()
        .filter_map(|token| {
            let role = capture_token_role(token.text)?;
            Some(MenuSyntaxInputSpan {
                range: token.range,
                role,
            })
        })
        .collect()
}

fn source_filter_head_span(raw: &str) -> Vec<MenuSyntaxInputSpan> {
    raw_tokens(raw, 0)
        .into_iter()
        .filter_map(|token| {
            if token.text.starts_with('"') || token.text.starts_with('\'') {
                return None;
            }
            let (head_start_offset, body) = token
                .text
                .strip_prefix('-')
                .map(|rest| (1, rest))
                .unwrap_or((0, token.text));
            let head = body.find(':').map(|colon_idx| &body[..=colon_idx])?;
            let descriptor = super::payload::SOURCE_HEAD_SPECS.iter().find(|spec| {
                spec.canonical.eq_ignore_ascii_case(head)
                    || spec
                        .short
                        .is_some_and(|short| short.eq_ignore_ascii_case(head))
            })?;
            if !descriptor.planned {
                return None;
            }
            let start = token.range.start + head_start_offset;
            Some(MenuSyntaxInputSpan {
                range: start..start + head.len(),
                role: MenuSyntaxFragmentRole::Prefix,
            })
        })
        .collect()
}

fn raw_tokens(raw: &str, start: usize) -> Vec<RawToken<'_>> {
    let bytes = raw.as_bytes();
    let mut out = Vec::new();
    let mut i = start.min(raw.len());
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let token_start = i;
        let mut in_quote: Option<u8> = None;
        while i < bytes.len() {
            let c = bytes[i];
            match in_quote {
                Some(q) if c == q => {
                    in_quote = None;
                    i += 1;
                }
                Some(_) => i += 1,
                None if c == b'"' || c == b'\'' => {
                    in_quote = Some(c);
                    i += 1;
                }
                None if c.is_ascii_whitespace() => break,
                None => i += 1,
            }
        }
        out.push(RawToken {
            text: &raw[token_start..i],
            range: token_start..i,
        });
    }
    out
}

fn capture_token_role(token: &str) -> Option<MenuSyntaxFragmentRole> {
    if typed_object_ref_token_is_resolved(token) {
        return Some(MenuSyntaxFragmentRole::ObjectRef);
    }
    if token.strip_prefix('#').is_some_and(|tag| !tag.is_empty()) {
        return Some(MenuSyntaxFragmentRole::Tag);
    }
    let lower = token.to_ascii_lowercase();
    if matches!(lower.as_str(), "p1" | "p2" | "p3" | "p4") {
        return Some(MenuSyntaxFragmentRole::Priority);
    }
    if token.starts_with("http://") || token.starts_with("https://") {
        return Some(MenuSyntaxFragmentRole::Url);
    }
    if let Some((key, value)) = token.split_once('=') {
        if !key.is_empty()
            && !value.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Some(MenuSyntaxFragmentRole::Kv);
        }
    }
    if let Some((key, value)) = token.split_once(':') {
        if !key.is_empty()
            && !value.is_empty()
            && !matches!(
                key.to_ascii_lowercase().as_str(),
                "due" | "at" | "start" | "end" | "url" | "for"
            )
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Some(MenuSyntaxFragmentRole::Kv);
        }
    }
    None
}

fn typed_object_ref_token_is_resolved(token: &str) -> bool {
    let Some(query) = token.strip_prefix('@') else {
        return false;
    };
    let Some((prefix, id)) = query.split_once(':') else {
        return false;
    };
    if id.trim().is_empty() {
        return false;
    }
    matches!(
        prefix.trim().to_ascii_lowercase().as_str(),
        "todo" | "todos" | "note" | "notes" | "link" | "links" | "snippet" | "snippets"
    )
}

fn find_fragment_range(raw: &str, start: usize, source: &str) -> Option<Range<usize>> {
    if source.is_empty() || start > raw.len() {
        return None;
    }
    let relative = raw[start..].find(source)?;
    let begin = start + relative;
    let end = begin + source.len();
    valid_utf8_range(raw, begin..end).then_some(begin..end)
}

fn normalize_input_spans(
    raw: &str,
    mut spans: Vec<MenuSyntaxInputSpan>,
    prefix: Option<Range<usize>>,
) -> Vec<MenuSyntaxInputSpan> {
    spans.retain(|span| valid_utf8_range(raw, span.range.clone()) && !span.range.is_empty());
    spans.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
            .then(role_rank(a.role).cmp(&role_rank(b.role)))
    });

    let mut out: Vec<MenuSyntaxInputSpan> = Vec::new();
    for span in spans {
        if span.role != MenuSyntaxFragmentRole::Prefix
            && prefix
                .as_ref()
                .is_some_and(|prefix| ranges_overlap(&span.range, prefix))
        {
            continue;
        }
        if out
            .last()
            .is_some_and(|previous| ranges_overlap(&span.range, &previous.range))
        {
            continue;
        }
        out.push(span);
    }
    out
}

fn role_rank(role: MenuSyntaxFragmentRole) -> u8 {
    match role {
        MenuSyntaxFragmentRole::Prefix => 0,
        MenuSyntaxFragmentRole::DateRange => 1,
        MenuSyntaxFragmentRole::Date => 2,
        MenuSyntaxFragmentRole::Duration => 3,
        MenuSyntaxFragmentRole::Recurrence => 4,
        MenuSyntaxFragmentRole::Priority => 5,
        MenuSyntaxFragmentRole::Url => 6,
        MenuSyntaxFragmentRole::ObjectRef => 7,
        MenuSyntaxFragmentRole::Tag => 8,
        MenuSyntaxFragmentRole::Kv => 9,
        MenuSyntaxFragmentRole::Unresolved => 10,
        MenuSyntaxFragmentRole::Subject => 11,
    }
}

fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

fn valid_utf8_range(raw: &str, range: Range<usize>) -> bool {
    range.start <= range.end
        && range.end <= raw.len()
        && raw.is_char_boundary(range.start)
        && raw.is_char_boundary(range.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::payload::{ArtifactKind, Predicate};

    #[test]
    fn default_mode_parses_as_none() {
        let mode = MenuSyntaxMode::default();
        assert_eq!(mode.raw(), "");
        assert_eq!(mode.parse(), &MenuSyntaxParse::None);
        assert!(!mode.is_menu_syntax_for(""));
    }

    #[test]
    fn from_input_stores_raw_and_parse() {
        let mode = MenuSyntaxMode::from_input(":type:script git");
        assert_eq!(mode.raw(), ":type:script git");
        match mode.parse() {
            MenuSyntaxParse::AdvancedQuery(q) => {
                assert_eq!(q.predicates, vec![Predicate::Type(ArtifactKind::Script)]);
                assert_eq!(q.free_text, "git");
            }
            other => panic!("expected AdvancedQuery, got {other:?}"),
        }
    }

    #[test]
    fn parse_for_guards_on_raw_mismatch() {
        let mode = MenuSyntaxMode::from_input(":type:script git");
        assert!(mode.parse_for(":type:script git").is_some());
        assert!(mode.parse_for("something else").is_none());
        assert!(mode.advanced_query_for(":type:script git").is_some());
        assert!(mode.advanced_query_for(":type:script gi").is_none());
    }

    #[test]
    fn capture_for_only_matches_raw() {
        let mode = MenuSyntaxMode::from_input(";todo Renew passport");
        assert!(mode.capture_for(";todo Renew passport").is_some());
        assert!(mode.capture_for(";todo Renew").is_none());
    }

    #[test]
    fn incomplete_exposes_hint_under_raw_guard() {
        let mode = MenuSyntaxMode::from_input(";");
        let hint = mode
            .incomplete_hint_for(";")
            .expect("bare plus is incomplete");
        assert!(hint.contains("todo"));
        assert!(mode.incomplete_hint_for("+x").is_none());
    }

    #[test]
    fn is_menu_syntax_for_false_on_plain_text() {
        let mode = MenuSyntaxMode::from_input("git deploy");
        assert!(!mode.is_menu_syntax_for("git deploy"));
        let mode_empty = MenuSyntaxMode::from_input("");
        assert!(!mode_empty.is_menu_syntax_for(""));
    }

    #[test]
    fn is_menu_syntax_for_true_on_incomplete_capture_and_query() {
        assert!(MenuSyntaxMode::from_input(":").is_menu_syntax_for(":"));
        assert!(MenuSyntaxMode::from_input(";").is_menu_syntax_for(";"));
        assert!(MenuSyntaxMode::from_input(";todo").is_menu_syntax_for(";todo"));
        assert!(MenuSyntaxMode::from_input(":type:script").is_menu_syntax_for(":type:script"));
        assert!(MenuSyntaxMode::from_input(">").is_menu_syntax_for(">"));
        assert!(MenuSyntaxMode::from_input(">deploy").is_menu_syntax_for(">deploy"));
    }

    #[test]
    fn capture_composer_starts_after_committed_target() {
        assert!(MenuSyntaxMode::from_input(";todo").capture_composer_owns_input_for(";todo"));
        assert!(MenuSyntaxMode::from_input(";note").capture_composer_owns_input_for(";note"));
        assert!(!MenuSyntaxMode::from_input(";to").capture_composer_owns_input_for(";to"));
        assert!(MenuSyntaxMode::from_input(";todo ").capture_composer_owns_input_for(";todo "));
        assert!(MenuSyntaxMode::from_input(";todo Take out")
            .capture_composer_owns_input_for(";todo Take out"));
    }

    #[test]
    fn registered_capture_target_owns_composer_after_boundary() {
        let targets = vec!["github".to_string()];
        let pending = MenuSyntaxMode::from_input_with_capture_targets("+github", &targets);
        assert!(pending.is_menu_syntax_for("+github"));
        assert!(pending.capture_composer_owns_input_for("+github"));

        let composing = MenuSyntaxMode::from_input_with_capture_targets("+github issue", &targets);
        assert!(composing.capture_composer_owns_input_for("+github issue"));
    }

    #[test]
    fn keyword_capture_alias_is_composer_after_colon() {
        assert!(MenuSyntaxMode::from_input("note:").capture_composer_owns_input_for("note:"));
        assert!(MenuSyntaxMode::from_input("note: Take out")
            .capture_composer_owns_input_for("note: Take out"));
    }

    #[test]
    fn free_text_for_search_strips_qualifiers() {
        let mode = MenuSyntaxMode::from_input(":type:script git deploy");
        assert_eq!(
            free_text_for_search(&mode, ":type:script git deploy"),
            "git deploy"
        );
    }

    #[test]
    fn free_text_for_search_strips_source_filter_anywhere() {
        let mode = MenuSyntaxMode::from_input("meeting :n");
        assert_eq!(free_text_for_search(&mode, "meeting :n"), "meeting");

        let mode = MenuSyntaxMode::from_input(":f project");
        assert_eq!(free_text_for_search(&mode, ":f project"), "project");
    }

    #[test]
    fn free_text_for_search_returns_raw_without_query_mode() {
        let mode = MenuSyntaxMode::from_input("hello world");
        assert_eq!(free_text_for_search(&mode, "hello world"), "hello world");
        let mode = MenuSyntaxMode::from_input(";todo Renew passport");
        assert_eq!(
            free_text_for_search(&mode, ";todo Renew passport"),
            ";todo Renew passport"
        );
    }

    #[test]
    fn free_text_for_search_guards_on_raw_mismatch() {
        let mode = MenuSyntaxMode::from_input(":type:script git");
        assert_eq!(
            free_text_for_search(&mode, "something else"),
            "something else"
        );
    }

    #[test]
    fn prefix_span_highlights_colon() {
        assert_eq!(prefix_span_for_input(":type:script git"), Some(0..12));
        assert_eq!(prefix_span_for_input(":typ"), Some(0..4));
        assert_eq!(prefix_span_for_input(":#work type:script"), Some(0..6));
        assert_eq!(prefix_span_for_input(":"), Some(0..1));
    }

    #[test]
    fn prefix_span_highlights_source_filter_head() {
        assert_eq!(prefix_span_for_input("f: project"), Some(0..2));
        assert_eq!(prefix_span_for_input("files:project"), Some(0..6));
        assert_eq!(prefix_span_for_input("c:sub"), Some(0..2));
        assert_eq!(prefix_span_for_input("todo: Renew passport"), Some(0..5));
    }

    #[test]
    fn prefix_span_highlights_plus_capture() {
        assert_eq!(prefix_span_for_input(";todo Renew passport"), Some(0..5));
        assert_eq!(prefix_span_for_input(";note"), Some(0..5));
        assert_eq!(prefix_span_for_input("+t"), Some(0..2));
        assert_eq!(prefix_span_for_input(";"), Some(0..1));
    }

    #[test]
    fn prefix_span_highlights_command_invocation_head() {
        assert_eq!(prefix_span_for_input(">"), Some(0..1));
        assert_eq!(prefix_span_for_input(">deploy"), Some(0..7));
        assert_eq!(prefix_span_for_input(">deploy -- prod"), Some(0..7));
    }

    #[test]
    fn prefix_span_highlights_keyword_capture() {
        assert_eq!(prefix_span_for_input("note: stuff"), Some(0..5));
    }

    #[test]
    fn prefix_span_none_on_plain_text() {
        assert_eq!(prefix_span_for_input(""), None);
        assert_eq!(prefix_span_for_input("git deploy"), None);
        assert_eq!(prefix_span_for_input("localhost:3000"), None);
        assert_eq!(prefix_span_for_input("not-a-target: stuff"), None);
    }

    #[test]
    fn prefix_span_none_on_unknown_plus_head() {
        assert_eq!(prefix_span_for_input("+github"), None);
        assert_eq!(prefix_span_for_input("+1"), None);
        assert_eq!(prefix_span_for_input("+react component"), None);
    }

    #[test]
    fn prefix_span_highlights_registered_capture_target() {
        let targets = vec!["github".to_string()];
        assert_eq!(
            prefix_span_for_input_with_targets("+github issue", &targets),
            Some(0..7)
        );
        assert_eq!(
            prefix_span_for_input_with_targets("github: issue", &targets),
            Some(0..7)
        );
    }

    #[test]
    fn input_spans_keep_prefix_span_backcompat() {
        let targets = vec!["github".to_string()];
        assert_eq!(
            input_spans_for_input_with_targets("+github issue", &targets)
                .first()
                .map(|span| span.range.clone()),
            prefix_span_for_input_with_targets("+github issue", &targets)
        );
    }

    #[test]
    fn input_spans_highlight_source_filter_heads_anywhere() {
        let raw = "budget f: c:sub -notes:done";
        let spans = input_spans_for_input(raw);
        let highlighted = spans
            .iter()
            .filter(|span| span.role == MenuSyntaxFragmentRole::Prefix)
            .map(|span| &raw[span.range.clone()])
            .collect::<Vec<_>>();
        assert_eq!(highlighted, vec!["f:", "c:", "notes:"]);
    }

    #[test]
    fn input_spans_highlight_mcal_range_duration_and_recurrence() {
        let range = input_spans_for_input(";mcal Lunch tomorrow at 12pm til 1pm");
        assert!(range
            .iter()
            .any(|span| span.role == MenuSyntaxFragmentRole::Prefix));
        assert!(range
            .iter()
            .any(|span| span.role == MenuSyntaxFragmentRole::DateRange));

        let duration = input_spans_for_input(";mcal Lunch tom 12pm for 30mins");
        assert!(duration
            .iter()
            .any(|span| span.role == MenuSyntaxFragmentRole::Duration));

        let recurrence = input_spans_for_input(";mcal Lunch every mon from 1 til 2");
        assert!(recurrence
            .iter()
            .any(|span| span.role == MenuSyntaxFragmentRole::Recurrence));
    }

    #[test]
    fn input_spans_highlight_todo_natural_date_phrase() {
        let raw = ";todo Eat lunch tom 3pm #food";
        let spans = input_spans_for_input(raw);
        assert!(spans.iter().any(|span| {
            span.role == MenuSyntaxFragmentRole::Date && &raw[span.range.clone()] == "tom 3pm"
        }));
    }

    #[test]
    fn input_spans_highlight_tag_and_kv_tokens() {
        let spans = input_spans_for_input(";mcal Lunch #work calendar=Work alarm:15");
        assert!(spans.iter().any(|span| {
            span.role == MenuSyntaxFragmentRole::Tag
                && &";mcal Lunch #work calendar=Work alarm:15"[span.range.clone()] == "#work"
        }));
        assert_eq!(
            spans
                .iter()
                .filter(|span| span.role == MenuSyntaxFragmentRole::Kv)
                .count(),
            2
        );
    }

    #[test]
    fn typed_object_ref_token_gets_object_ref_role_not_kv() {
        let raw = ";snippet update @snippet:fetch-json -- const value = 1";
        let spans = input_spans_for_input(raw);
        let object_ref = spans
            .iter()
            .find(|span| span.role == MenuSyntaxFragmentRole::ObjectRef)
            .expect("typed object ref span");
        assert_eq!(&raw[object_ref.range.clone()], "@snippet:fetch-json");
        assert!(
            !spans.iter().any(|span| {
                span.role == MenuSyntaxFragmentRole::Kv
                    && &raw[span.range.clone()] == "@snippet:fetch-json"
            }),
            "typed object ref should not be decorated as kv"
        );
    }

    #[test]
    fn input_spans_do_not_overlap_prefix() {
        let spans = input_spans_for_input(";mcal #tag");
        let prefix = spans
            .iter()
            .find(|span| span.role == MenuSyntaxFragmentRole::Prefix)
            .expect("prefix");
        assert!(spans.iter().all(|span| {
            span.role == MenuSyntaxFragmentRole::Prefix
                || !ranges_overlap(&span.range, &prefix.range)
        }));
    }

    #[test]
    fn input_spans_are_valid_utf8_byte_boundaries() {
        let raw = ";mcal Café tomorrow #déjà calendar=Work";
        for span in input_spans_for_input(raw) {
            assert!(raw.is_char_boundary(span.range.start), "{span:?}");
            assert!(raw.is_char_boundary(span.range.end), "{span:?}");
        }
    }

    #[test]
    fn input_spans_empty_for_plain_text() {
        assert!(input_spans_for_input("hello world #not-capture").is_empty());
    }

    #[test]
    fn source_head_span_does_not_claim_home_path() {
        assert!(
            input_spans_for_input("~/").is_empty(),
            "`~/` is a home-path/file-search handoff, not source-head chrome"
        );
        assert_eq!(prefix_span_for_input("~/"), None);
    }

    #[test]
    fn source_head_span_clears_after_f_head_replaced_by_home_path() {
        let f_spans = input_spans_for_input("f: xy");
        assert_eq!(f_spans.len(), 1);
        assert_eq!(f_spans[0].range, 0..2);
        assert_eq!(&"f: xy"[f_spans[0].range.clone()], "f:");

        assert!(
            input_spans_for_input("~/").is_empty(),
            "replacing `f:` with `~/` must produce an empty decoration set"
        );
    }

    #[test]
    fn source_head_spans_clear_for_plain_path_and_text_replacements() {
        let replacements = ["~/", "/tmp", "plain text"];

        for spec in crate::menu_syntax::payload::SOURCE_HEAD_SPECS
            .iter()
            .filter(|spec| spec.planned)
        {
            for head in [Some(spec.canonical), spec.short].into_iter().flatten() {
                let source_input = format!("{head} xy");
                assert!(
                    !input_spans_for_input(&source_input).is_empty(),
                    "{head} should produce source-head chrome before replacement"
                );

                for replacement in replacements {
                    assert!(
                        input_spans_for_input(replacement).is_empty(),
                        "{head} -> {replacement} must clear input chrome"
                    );
                }
            }
        }
    }

    #[test]
    fn power_syntax_spans_clear_for_plain_path_and_text_replacements() {
        for source_input in [";todo capture", ">command arg"] {
            assert!(
                !input_spans_for_input(source_input).is_empty(),
                "{source_input} should produce power-syntax chrome before replacement"
            );
        }

        for replacement in ["~/", "/tmp", "plain text"] {
            assert!(
                input_spans_for_input(replacement).is_empty(),
                "{replacement} must clear prior power-syntax input chrome"
            );
        }
    }
}
