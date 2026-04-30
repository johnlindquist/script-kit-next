use super::payload::{
    is_known_capture_target, CaptureAlias, CaptureInvocation, DatePhrase, DateRole, IncompleteKind,
    IncompleteSyntax, KNOWN_CAPTURE_TARGETS,
};

pub enum CaptureParse {
    Ok(CaptureInvocation),
    Incomplete(IncompleteSyntax),
}

pub fn parse_capture(input: &str) -> CaptureParse {
    parse_capture_with_targets(input, &[])
}

pub fn parse_capture_with_targets(input: &str, registered_targets: &[String]) -> CaptureParse {
    let (target, body_start, alias_form) = match split_target(input) {
        Some(v) => v,
        None => {
            return CaptureParse::Incomplete(IncompleteSyntax {
                kind: IncompleteKind::BareCapturePrefix,
                hint: format!(
                    "Choose a capture target: {}",
                    KNOWN_CAPTURE_TARGETS.join(", ")
                ),
            });
        }
    };

    if !is_capture_target_registered(&target, registered_targets) {
        return CaptureParse::Incomplete(IncompleteSyntax {
            kind: IncompleteKind::UnknownCaptureTarget(target.clone()),
            hint: format!(
                "Unknown capture target '{target}'. Known: {}",
                KNOWN_CAPTURE_TARGETS.join(", ")
            ),
        });
    }

    let raw = input.to_string();
    let rest = input[body_start..].trim_start();
    let tokens = scan_tokens(rest, body_start + leading_whitespace(&input[body_start..]));

    let mut body_parts: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    let mut priority: Option<u8> = None;
    let mut url: Option<String> = None;
    let mut duration: Option<String> = None;
    let mut kv: Vec<(String, String)> = Vec::new();
    let mut date_phrases: Vec<DatePhrase> = Vec::new();

    for tok in tokens {
        if let Some(tag) = tok.text.strip_prefix('#') {
            if !tag.is_empty() {
                tags.push(tag.to_string());
                continue;
            }
        }

        if let Some(p) = parse_priority(&tok.text) {
            priority = Some(p);
            continue;
        }

        if is_url_like(&tok.text) {
            url = Some(tok.text.clone());
            continue;
        }

        if let Some((key, value)) = split_colon_key(&tok.text) {
            let key_lower = key.to_ascii_lowercase();
            match key_lower.as_str() {
                "due" | "at" | "start" | "end" => {
                    let role = date_role(&key_lower);
                    let unquoted = unquote(&value);
                    let phrase_start = tok.span.0 + key.len() + 1;
                    let phrase_end = tok.span.1;
                    date_phrases.push(DatePhrase {
                        role,
                        source: unquoted,
                        source_span: (phrase_start, phrase_end),
                    });
                    continue;
                }
                "url" => {
                    url = Some(unquote(&value));
                    continue;
                }
                "for" => {
                    duration = Some(unquote(&value));
                    continue;
                }
                _ => {}
            }
        }

        if let Some((key, value)) = split_eq_key(&tok.text) {
            kv.push((key.to_string(), unquote(&value)));
            continue;
        }

        body_parts.push(tok.text);
    }

    CaptureParse::Ok(CaptureInvocation {
        target,
        alias_form,
        body: body_parts.join(" "),
        tags,
        priority,
        url,
        duration,
        kv,
        date_phrases,
        raw,
    })
}

pub fn is_capture_target_registered(target: &str, registered_targets: &[String]) -> bool {
    is_known_capture_target(target)
        || registered_targets
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(target))
}

fn split_target(input: &str) -> Option<(String, usize, CaptureAlias)> {
    if let Some(rest) = input.strip_prefix(';').or_else(|| input.strip_prefix('+')) {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        if end == 0 {
            return None;
        }
        let target = rest[..end].to_ascii_lowercase();
        return Some((target, 1 + end, CaptureAlias::Plus));
    }

    let colon_idx = input.find(':')?;
    let head = &input[..colon_idx];
    if head.is_empty() || head.contains(char::is_whitespace) {
        return None;
    }
    if !head
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    Some((
        head.to_ascii_lowercase(),
        colon_idx + 1,
        CaptureAlias::Keyword,
    ))
}

fn leading_whitespace(s: &str) -> usize {
    s.bytes().take_while(|b| b.is_ascii_whitespace()).count()
}

#[derive(Debug, Clone)]
struct SpannedToken {
    text: String,
    span: (usize, usize),
}

fn scan_tokens(input: &str, base_offset: usize) -> Vec<SpannedToken> {
    let mut tokens: Vec<SpannedToken> = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
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
        let text = input[start..i].to_string();
        tokens.push(SpannedToken {
            text,
            span: (base_offset + start, base_offset + i),
        });
    }
    tokens
}

fn parse_priority(tok: &str) -> Option<u8> {
    let lower = tok.to_ascii_lowercase();
    if lower.len() != 2 {
        return None;
    }
    if !lower.starts_with('p') {
        return None;
    }
    let n = lower[1..].parse::<u8>().ok()?;
    if (1..=4).contains(&n) {
        Some(n)
    } else {
        None
    }
}

fn is_url_like(tok: &str) -> bool {
    tok.starts_with("http://") || tok.starts_with("https://")
}

fn split_colon_key(tok: &str) -> Option<(&str, String)> {
    let idx = tok.find(':')?;
    let key = &tok[..idx];
    if key.is_empty() || key.contains(char::is_whitespace) {
        return None;
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return None;
    }
    Some((key, tok[idx + 1..].to_string()))
}

fn split_eq_key(tok: &str) -> Option<(&str, String)> {
    let idx = tok.find('=')?;
    let key = &tok[..idx];
    if key.is_empty() || key.contains(char::is_whitespace) {
        return None;
    }
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    Some((key, tok[idx + 1..].to_string()))
}

fn date_role(key: &str) -> DateRole {
    match key {
        "due" => DateRole::Due,
        "at" => DateRole::At,
        "start" => DateRole::Start,
        "end" => DateRole::End,
        _ => DateRole::Inferred,
    }
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
    use crate::menu_syntax::payload::CaptureAlias;

    fn ok(input: &str) -> CaptureInvocation {
        match parse_capture(input) {
            CaptureParse::Ok(inv) => inv,
            CaptureParse::Incomplete(s) => panic!("expected ok, got incomplete: {s:?}"),
        }
    }

    fn incomplete(input: &str) -> IncompleteSyntax {
        match parse_capture(input) {
            CaptureParse::Incomplete(s) => s,
            CaptureParse::Ok(inv) => panic!("expected incomplete, got ok: {inv:?}"),
        }
    }

    #[test]
    fn parses_plus_todo_with_body_and_tags() {
        let inv = ok(";todo Renew passport #errands p1");
        assert_eq!(inv.target, "todo");
        assert_eq!(inv.alias_form, CaptureAlias::Plus);
        assert_eq!(inv.body, "Renew passport");
        assert_eq!(inv.tags, vec!["errands".to_string()]);
        assert_eq!(inv.priority, Some(1));
    }

    #[test]
    fn parses_keyword_alias() {
        let inv = ok("todo: Email invoices #finance");
        assert_eq!(inv.target, "todo");
        assert_eq!(inv.alias_form, CaptureAlias::Keyword);
        assert_eq!(inv.body, "Email invoices");
        assert_eq!(inv.tags, vec!["finance".to_string()]);
    }

    #[test]
    fn parses_explicit_due_key() {
        let inv = ok(";todo Renew passport due:\"tomorrow 3pm\" #errands");
        assert_eq!(inv.date_phrases.len(), 1);
        assert_eq!(inv.date_phrases[0].role, DateRole::Due);
        assert_eq!(inv.date_phrases[0].source, "tomorrow 3pm");
        assert!(!inv.body.contains("due:"));
    }

    #[test]
    fn parses_calendar_start_and_duration() {
        let inv = ok(";cal Lunch start:next-friday-noon for:45m #work");
        assert_eq!(inv.target, "cal");
        assert_eq!(inv.date_phrases.len(), 1);
        assert_eq!(inv.date_phrases[0].role, DateRole::Start);
        assert_eq!(inv.duration, Some("45m".to_string()));
        assert_eq!(inv.tags, vec!["work".to_string()]);
        assert_eq!(inv.body, "Lunch");
    }

    #[test]
    fn parses_url_both_bare_and_key() {
        let inv = ok(";link https://zed.dev #rust #gpui");
        assert_eq!(inv.url.as_deref(), Some("https://zed.dev"));
        assert_eq!(inv.tags, vec!["rust".to_string(), "gpui".to_string()]);

        let inv2 = ok(";link url:https://example.com title=\"Menu syntax\" #research");
        assert_eq!(inv2.url.as_deref(), Some("https://example.com"));
        assert_eq!(
            inv2.kv,
            vec![("title".to_string(), "Menu syntax".to_string())]
        );
    }

    #[test]
    fn priority_must_be_p1_p4() {
        let inv = ok(";todo task p5");
        assert_eq!(inv.priority, None);
        assert!(inv.body.contains("p5"));

        let inv2 = ok(";todo task p2");
        assert_eq!(inv2.priority, Some(2));
    }

    #[test]
    fn bare_plus_is_incomplete() {
        let s = incomplete("+");
        assert!(matches!(s.kind, IncompleteKind::BareCapturePrefix));
    }

    #[test]
    fn unknown_target_is_incomplete() {
        let s = incomplete("+xyz whatever");
        assert!(matches!(
            s.kind,
            IncompleteKind::UnknownCaptureTarget(ref t) if t == "xyz"
        ));
    }

    #[test]
    fn keyword_alias_with_unknown_head_is_not_capture() {
        // `todo ` has no colon → this module would never be called.
        // But a colon without a known head (e.g. `xyz:`) must not parse as capture.
        match parse_capture("xyz:stuff") {
            CaptureParse::Incomplete(s) => {
                assert!(matches!(s.kind, IncompleteKind::UnknownCaptureTarget(_)))
            }
            CaptureParse::Ok(inv) => panic!("should not parse as ok: {inv:?}"),
        }
    }

    #[test]
    fn trailing_hash_is_not_a_tag() {
        let inv = ok(";note Decision #menu-syntax #");
        assert_eq!(inv.tags, vec!["menu-syntax".to_string()]);
        assert!(inv.body.ends_with('#'));
    }
}
