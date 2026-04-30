use crate::menu_syntax::date::ResolvedDuration;
use crate::menu_syntax::fragments::{
    MenuSyntaxFragment, MenuSyntaxFragmentRole, MenuSyntaxFragmentStatus,
};
use crate::menu_syntax::nl_phrase::{ConsumedTokens, NlToken, ParseHit};

#[derive(Debug, Clone)]
pub(super) struct ResolvedDurationPhrase {
    pub duration: ResolvedDuration,
}

pub(super) fn parse_duration(
    tokens: &[NlToken],
    consumed: &ConsumedTokens,
) -> Option<ParseHit<ResolvedDurationPhrase>> {
    for idx in 0..tokens.len().saturating_sub(1) {
        if consumed.is_consumed(idx) || tokens[idx].lower != "for" {
            continue;
        }
        let (dur_start_idx, dur_end_idx, seconds) =
            if let Some(seconds) = parse_duration_token(&tokens[idx + 1].lower) {
                (idx + 1, idx + 1, seconds)
            } else if let Some(unit) = tokens.get(idx + 2) {
                let seconds = parse_duration_parts(&tokens[idx + 1].lower, &unit.lower)?;
                (idx + 1, idx + 2, seconds)
            } else {
                continue;
            };
        if (dur_start_idx..=dur_end_idx).any(|i| consumed.is_consumed(i)) {
            continue;
        }
        let span = (tokens[dur_start_idx].span.0, tokens[dur_end_idx].span.1);
        let source = span_source(tokens, span);
        let duration = ResolvedDuration {
            source: source.clone(),
            source_span: span,
            seconds,
            minutes: seconds / 60,
            iso8601: duration_iso8601(seconds),
        };
        let mut consumed_tokens = vec![idx];
        consumed_tokens.extend(dur_start_idx..=dur_end_idx);
        return Some(ParseHit {
            value: ResolvedDurationPhrase { duration },
            fragment: fragment(MenuSyntaxFragmentRole::Duration, source, span),
            consumed: consumed_tokens,
        });
    }
    None
}

pub(super) fn parse_relative_duration_after_in(
    tokens: &[NlToken],
    start_idx: usize,
) -> Option<ResolvedDurationPhrase> {
    let first = tokens.get(start_idx)?;
    let (end_idx, seconds) = if let Some(seconds) = parse_duration_token(&first.lower) {
        (start_idx, seconds)
    } else {
        let unit = tokens.get(start_idx + 1)?;
        (
            start_idx + 1,
            parse_duration_parts(&first.lower, &unit.lower)?,
        )
    };
    let span = (first.span.0, tokens[end_idx].span.1);
    let source = span_source(tokens, span);
    Some(ResolvedDurationPhrase {
        duration: ResolvedDuration {
            source,
            source_span: span,
            seconds,
            minutes: seconds / 60,
            iso8601: duration_iso8601(seconds),
        },
    })
}

pub(super) fn parse_duration_token(source: &str) -> Option<u32> {
    let digit_len = source.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digit_len == 0 {
        return None;
    }
    let amount = source[..digit_len].parse::<u32>().ok()?;
    let unit = &source[digit_len..];
    unit_seconds(unit).map(|seconds| amount.saturating_mul(seconds))
}

fn parse_duration_parts(amount: &str, unit: &str) -> Option<u32> {
    let amount = amount.parse::<u32>().ok()?;
    unit_seconds(unit).map(|seconds| amount.saturating_mul(seconds))
}

fn unit_seconds(unit: &str) -> Option<u32> {
    match unit {
        "s" | "sec" | "secs" | "second" | "seconds" => Some(1),
        "m" | "min" | "mins" | "minute" | "minutes" => Some(60),
        "h" | "hr" | "hrs" | "hour" | "hours" => Some(60 * 60),
        "d" | "day" | "days" => Some(24 * 60 * 60),
        "w" | "wk" | "wks" | "week" | "weeks" => Some(7 * 24 * 60 * 60),
        _ => None,
    }
}

pub(super) fn duration_iso8601(seconds: u32) -> String {
    if seconds % 86_400 == 0 {
        format!("P{}D", seconds / 86_400)
    } else if seconds % 3600 == 0 {
        format!("PT{}H", seconds / 3600)
    } else if seconds % 60 == 0 {
        format!("PT{}M", seconds / 60)
    } else {
        format!("PT{}S", seconds)
    }
}

pub(super) fn unresolved_fragment(source: &str, source_span: (usize, usize)) -> MenuSyntaxFragment {
    MenuSyntaxFragment {
        role: MenuSyntaxFragmentRole::Unresolved,
        source: source.to_string(),
        source_span,
        status: MenuSyntaxFragmentStatus::Unresolved,
    }
}

fn fragment(
    role: MenuSyntaxFragmentRole,
    source: String,
    source_span: (usize, usize),
) -> MenuSyntaxFragment {
    MenuSyntaxFragment {
        role,
        source,
        source_span,
        status: MenuSyntaxFragmentStatus::Resolved,
    }
}

fn span_source(tokens: &[NlToken], span: (usize, usize)) -> String {
    tokens
        .iter()
        .find(|token| token.span.0 <= span.0 && token.span.1 >= span.1)
        .map(|token| token.text.clone())
        .unwrap_or_else(|| {
            tokens
                .iter()
                .filter(|token| token.span.0 >= span.0 && token.span.1 <= span.1)
                .map(|token| token.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::nl_phrase::{tokenize, ConsumedTokens};

    #[test]
    fn for_30mins_compact_duration() {
        let tokens = tokenize("Lunch for 30mins");
        let hit = parse_duration(&tokens, &ConsumedTokens::new(tokens.len())).expect("duration");
        assert_eq!(hit.value.duration.seconds, 1800);
        assert_eq!(hit.value.duration.iso8601, "PT30M");
    }

    #[test]
    fn for_30_minutes_spaced_duration() {
        let tokens = tokenize("Lunch for 30 minutes");
        let hit = parse_duration(&tokens, &ConsumedTokens::new(tokens.len())).expect("duration");
        assert_eq!(hit.value.duration.seconds, 1800);
        assert_eq!(hit.value.duration.source, "30 minutes");
    }

    #[test]
    fn in_30_minutes_duration_parts_reusable_for_relative_date() {
        let tokens = tokenize("in 30 minutes");
        let duration = parse_relative_duration_after_in(&tokens, 1).expect("duration");
        assert_eq!(duration.duration.seconds, 1800);
        assert_eq!(duration.duration.minutes, 30);
    }
}
